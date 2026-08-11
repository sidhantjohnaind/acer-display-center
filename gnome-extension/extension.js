import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Slider from 'resource:///org/gnome/shell/ui/slider.js';
import St from 'gi://St';
import GLib from 'gi://GLib';
import Clutter from 'gi://Clutter';
import GObject from 'gi://GObject';

/* ------------------------------------------------------------------ */
/*  CLI helper                                                          */
/* ------------------------------------------------------------------ */
function execCli(cmd) {
    GLib.spawn_command_line_async(`/usr/local/bin/amctl ${cmd}`);
}

function getInitialState() {
    try {
        let [res, stdout] = GLib.spawn_command_line_sync('/usr/local/bin/amctl info --json');
        if (res && stdout) {
            let str = new TextDecoder().decode(stdout);
            let parsed = JSON.parse(str);
            if (parsed?.current_values) {
                let cv = parsed.current_values;
                let modeVal = cv.display_mode?.current ?? 1;
                let modeName = {
                    0: 'User', 1: 'Standard', 2: 'ECO', 3: 'Graphics',
                    5: 'Action', 6: 'Racing', 7: 'Sports', 11: 'HDR'
                }[modeVal] ?? 'Standard';
                return {
                    brightness: cv.brightness?.current ?? 80,
                    contrast:   cv.contrast?.current   ?? 50,
                    mode_name:  modeName,
                };
            }
        }
    } catch (_) { /* ignore */ }
    return { brightness: 80, contrast: 50, mode_name: 'Standard' };
}

/* ------------------------------------------------------------------ */
/*  Native-looking slider widget                                        */
/*  Mirrors how GNOME 46's QuickSlider is built internally             */
/* ------------------------------------------------------------------ */
const AcerSlider = GObject.registerClass(
class AcerSlider extends St.Widget {
    _init(iconName, initialValue, onChange) {
        super._init({
            style_class: 'quick-slider',
            layout_manager: new Clutter.BinLayout(),
            x_expand: true,
        });

        // Inner box — same structure as native QuickSlider
        this._box = new St.BoxLayout({ x_expand: true });
        this.add_child(this._box);

        this._icon = new St.Icon({
            icon_name: iconName,
            style_class: 'quick-slider-icon',
            y_align: Clutter.ActorAlign.CENTER,
        });
        this._box.add_child(this._icon);

        this._slider = new Slider.Slider(initialValue / 100.0);
        this._slider.x_expand = true;
        this._box.add_child(this._slider);

        this._timeout = 0;
        this._slider.connect('notify::value', () => {
            let val = Math.round(this._slider.value * 100);
            if (this._timeout) GLib.source_remove(this._timeout);
            this._timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 120, () => {
                onChange(val);
                this._timeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }

    destroy() {
        if (this._timeout) {
            GLib.source_remove(this._timeout);
            this._timeout = 0;
        }
        super.destroy();
    }
});

/* ------------------------------------------------------------------ */
/*  Extension                                                           */
/* ------------------------------------------------------------------ */
export default class AcerMonitorExtension extends Extension {
    enable() {
        let state = getInitialState();
        let qs    = Main.panel.statusArea.quickSettings;
        let grid  = qs._grid;          // The actual St.Widget grid
        let lm    = grid.layout_manager;

        this._inGrid = [];
        this._inMenu = [];

        // --- Brightness slider (inserted at grid position 0) ---
        this._bright = new AcerSlider(
            'display-brightness-symbolic',
            state.brightness,
            val => execCli(`brightness ${val}`)
        );
        grid.insert_child_at_index(this._bright, 0);
        try { lm.set_column_span(this._bright, 2); } catch (_) {}
        this._inGrid.push(this._bright);

        // --- Contrast slider (inserted at grid position 1) ---
        this._contrast = new AcerSlider(
            'display-symbolic',
            state.contrast,
            val => execCli(`contrast ${val}`)
        );
        grid.insert_child_at_index(this._contrast, 1);
        try { lm.set_column_span(this._contrast, 2); } catch (_) {}
        this._inGrid.push(this._contrast);

        // --- Preset toggle (native PopupSubMenuMenuItem pill) ---
        this._preset = new PopupMenu.PopupSubMenuMenuItem(
            `Preset: ${state.mode_name}`, true
        );
        this._preset.icon.icon_name = 'video-display-symbolic';

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

        for (let m of MODES) {
            let item = new PopupMenu.PopupMenuItem(m.label);
            item.connect('activate', () => {
                execCli(m.cmd);
                this._preset.label.text = `Preset: ${m.short}`;
            });
            this._preset.menu.addMenuItem(item);
        }

        // Add preset to the QuickSettings popup menu (shows below grid, native behaviour)
        qs.menu.addMenuItem(this._preset, 0);
        this._inMenu.push(this._preset);
    }

    disable() {
        // Remove grid items (St.Widget — remove from parent)
        for (let w of this._inGrid ?? []) {
            w.destroy();
        }
        this._inGrid = [];

        // Remove menu items
        for (let w of this._inMenu ?? []) {
            w.destroy();
        }
        this._inMenu = [];
    }
}
