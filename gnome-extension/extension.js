import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Slider from 'resource:///org/gnome/shell/ui/slider.js';
import St from 'gi://St';
import GLib from 'gi://GLib';
import Clutter from 'gi://Clutter';
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
/*  Slider row — styled to match native GNOME quick-settings look      */
/*  (cannot use qs._grid due to Ubuntu-patched QuickSettingsLayout)    */
/* ------------------------------------------------------------------ */
const AcerSlider = GObject.registerClass(
class AcerSlider extends St.BoxLayout {
    _init(iconName, initialValue, onChange) {
        super._init({
            // Match the panel's horizontal padding (18px each side)
            // and a bottom gap equal to the grid row spacing (12px)
            style: `
                padding: 0 18px 12px 18px;
                spacing: 6px;
            `,
            x_expand: true,
        });

        this._icon = new St.Icon({
            icon_name: iconName,
            icon_size: 16,
            style_class: 'quick-slider-icon',
            y_align: Clutter.ActorAlign.CENTER,
        });
        this.add_child(this._icon);

        // slider-bin wrapper (matches .quick-slider .slider-bin CSS)
        let bin = new St.Bin({
            style: 'padding: 6px; border-radius: 999px;',
            x_expand: true,
            y_align: Clutter.ActorAlign.CENTER,
        });
        this._slider = new Slider.Slider(initialValue / 100.0);
        bin.set_child(this._slider);
        this.add_child(bin);

        this._label = new St.Label({
            text: `${initialValue}%`,
            y_align: Clutter.ActorAlign.CENTER,
            style: 'min-width: 2.8em; text-align: right; font-size: 0.9em;',
        });
        this.add_child(this._label);

        this._timeout = 0;
        this._slider.connect('notify::value', () => {
            let val = Math.round(this._slider.value * 100);
            this._label.text = `${val}%`;
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
export default class AcerMonitorExtension extends Extension {
    enable() {
        let state = getInitialState();
        let qs    = Main.panel.statusArea.quickSettings;
        let box   = qs.menu.box;   // Safe: St.BoxLayout, no broken layout manager

        this._items = [];

        // Find where the tile grid lives inside the menu box.
        // Insert our sliders just BEFORE it — anything above the grid
        // (system section, power button) stays untouched at the top.
        let gridIndex = 0;
        let boxChildren = box.get_children();
        for (let i = 0; i < boxChildren.length; i++) {
            if (boxChildren[i] === qs._grid) {
                gridIndex = i;
                break;
            }
        }

        // === Brightness slider ===
        let bright = new AcerSlider(
            'display-brightness-symbolic',
            state.brightness,
            val => execCli(`brightness ${val}`)
        );
        box.insert_child_at_index(bright, gridIndex);
        this._items.push(bright);

        // === Contrast slider ===
        let contrast = new AcerSlider(
            'display-symbolic',
            state.contrast,
            val => execCli(`contrast ${val}`)
        );
        box.insert_child_at_index(contrast, gridIndex + 1);
        this._items.push(contrast);

        // === Preset submenu (native pill via PopupMenu) ===
        let preset = new PopupMenu.PopupSubMenuMenuItem(
            `Preset: ${state.mode_name}`, true
        );
        preset.icon.icon_name = 'video-display-symbolic';

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
                preset.label.text = `Preset: ${m.short}`;
            });
            preset.menu.addMenuItem(item);
        }

        qs.menu.addMenuItem(preset, 0);
        this._items.push(preset);
    }

    disable() {
        for (let w of this._items ?? []) {
            w.destroy();
        }
        this._items = [];
    }
}
