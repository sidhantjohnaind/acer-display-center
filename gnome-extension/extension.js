import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {
    QuickSlider,
    QuickMenuToggle,
} from 'resource:///org/gnome/shell/ui/quickSettings.js';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';

/* ------------------------------------------------------------------ */
function execCli(cmd) {
    GLib.spawn_command_line_async(`/usr/local/bin/amctl ${cmd}`);
}

function getInitialState() {
    try {
        let [res, stdout] = GLib.spawn_command_line_sync('/usr/local/bin/amctl info --json');
        if (res && stdout) {
            let str = new TextDecoder().decode(stdout);
            let p = JSON.parse(str);
            if (p?.current_values) {
                let cv = p.current_values;
                let modeVal = cv.display_mode?.current ?? 1;
                let modeName = {
                    0: 'User', 1: 'Standard', 2: 'ECO', 3: 'Graphics',
                    5: 'Action', 6: 'Racing', 7: 'Sports', 11: 'HDR',
                }[modeVal] ?? 'Standard';
                return {
                    brightness: cv.brightness?.current ?? 80,
                    contrast:   cv.contrast?.current   ?? 50,
                    modeName,
                };
            }
        }
    } catch (_) { /* ignore */ }
    return { brightness: 80, contrast: 50, modeName: 'Standard' };
}

/* ------------------------------------------------------------------ */
/*  Brightness slider — subclasses the native QuickSlider              */
/* ------------------------------------------------------------------ */
const AcerBrightnessSlider = GObject.registerClass(
class AcerBrightnessSlider extends QuickSlider {
    _init(initialValue) {
        super._init({ iconName: 'display-brightness-symbolic' });
        // Hide the "open menu" arrow — we don't need a submenu
        this.menuEnabled = false;

        this.slider.value = initialValue / 100.0;
        this._timeout = 0;
        this.slider.connect('notify::value', () => {
            const val = Math.round(this.slider.value * 100);
            if (this._timeout) GLib.source_remove(this._timeout);
            this._timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 120, () => {
                execCli(`brightness ${val}`);
                this._timeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }

    destroy() {
        if (this._timeout) { GLib.source_remove(this._timeout); this._timeout = 0; }
        super.destroy();
    }
});

/* ------------------------------------------------------------------ */
/*  Contrast slider — subclasses the native QuickSlider                */
/* ------------------------------------------------------------------ */
const AcerContrastSlider = GObject.registerClass(
class AcerContrastSlider extends QuickSlider {
    _init(initialValue) {
        super._init({ iconName: 'display-symbolic' });
        this.menuEnabled = false;

        this.slider.value = initialValue / 100.0;
        this._timeout = 0;
        this.slider.connect('notify::value', () => {
            const val = Math.round(this.slider.value * 100);
            if (this._timeout) GLib.source_remove(this._timeout);
            this._timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 120, () => {
                execCli(`contrast ${val}`);
                this._timeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }

    destroy() {
        if (this._timeout) { GLib.source_remove(this._timeout); this._timeout = 0; }
        super.destroy();
    }
});

/* ------------------------------------------------------------------ */
/*  Preset toggle — subclasses the native QuickMenuToggle pill         */
/* ------------------------------------------------------------------ */
const MODES = [
    { label: 'User Mode',       short: 'User',     cmd: 'preset user'     },
    { label: 'Standard Mode',   short: 'Standard', cmd: 'preset standard' },
    { label: 'ECO Power Saver', short: 'ECO',      cmd: 'preset eco'      },
    { label: 'Graphics Mode',   short: 'Graphics', cmd: 'preset graphics' },
    { label: 'HDR Mode',        short: 'HDR',      cmd: 'preset hdr'      },
    { label: 'Action Gaming',   short: 'Action',   cmd: 'preset action'   },
    { label: 'Racing Mode',     short: 'Racing',   cmd: 'preset racing'   },
    { label: 'Sports Mode',     short: 'Sports',   cmd: 'preset sports'   },
];

const AcerPresetToggle = GObject.registerClass(
class AcerPresetToggle extends QuickMenuToggle {
    _init(initialModeName) {
        super._init({
            title: 'Acer Preset',
            subtitle: initialModeName,
            iconName: 'video-display-symbolic',
        });

        this.menu.setHeader('video-display-symbolic', 'Display Preset', initialModeName);

        const section = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(section);

        for (const m of MODES) {
            const item = new PopupMenu.PopupMenuItem(m.label);
            item.connect('activate', () => {
                execCli(m.cmd);
                this.subtitle = m.short;
                this.menu.setHeader('video-display-symbolic', 'Display Preset', m.short);
            });
            section.addMenuItem(item);
        }
    }
});

/* ------------------------------------------------------------------ */
/*  Extension                                                           */
/* ------------------------------------------------------------------ */
export default class AcerMonitorExtension extends Extension {
    enable() {
        const state = getInitialState();
        const sysMenu = Main.panel.statusArea.quickSettings.menu;

        this._items = [];

        // Build our items
        const preset   = new AcerPresetToggle(state.modeName);
        const contrast = new AcerContrastSlider(state.contrast);
        const bright   = new AcerBrightnessSlider(state.brightness);

        // Insert before the CURRENT first item so order ends up:
        //   Brightness → Contrast → Preset → [native volume, tiles, power...]
        //
        // We insert in REVERSE order (each goes "before current first"):
        const firstItem = sysMenu.getFirstItem();
        sysMenu.insertItemBefore(preset,   firstItem, 1);
        sysMenu.insertItemBefore(contrast, preset,    1);
        sysMenu.insertItemBefore(bright,   contrast,  1);

        this._items.push(bright, contrast, preset);
    }

    disable() {
        for (const w of this._items ?? [])
            w.destroy();
        this._items = [];
    }
}
