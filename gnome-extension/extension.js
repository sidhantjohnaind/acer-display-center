import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Slider from 'resource:///org/gnome/shell/ui/slider.js';
import St from 'gi://St';
import GLib from 'gi://GLib';
import Clutter from 'gi://Clutter';

function execCli(cmd) {
    GLib.spawn_command_line_async(`/usr/local/bin/amctl ${cmd}`);
}

function getInitialState() {
    try {
        let [res, stdout] = GLib.spawn_command_line_sync('/usr/local/bin/amctl info --json');
        if (res && stdout) {
            let str = new TextDecoder().decode(stdout);
            let parsed = JSON.parse(str);
            if (parsed && parsed.current_values) {
                let modeVal = parsed.current_values.display_mode
                    ? parsed.current_values.display_mode.current : 1;
                let modeName = {
                    0: 'User', 1: 'Standard', 2: 'ECO', 3: 'Graphics',
                    5: 'Action', 6: 'Racing', 7: 'Sports', 11: 'HDR'
                }[modeVal] || `Mode ${modeVal}`;
                return {
                    brightness: parsed.current_values.brightness
                        ? parsed.current_values.brightness.current : 80,
                    contrast: parsed.current_values.contrast
                        ? parsed.current_values.contrast.current : 50,
                    mode_name: modeName,
                };
            }
        }
    } catch (_) { /* ignore */ }
    return { brightness: 80, contrast: 50, mode_name: 'Standard' };
}

function makeSliderRow(iconName, value, onChange) {
    let row = new St.BoxLayout({
        style_class: 'quick-slider',
        reactive: true,
        x_expand: true,
    });

    let icon = new St.Icon({
        icon_name: iconName,
        style_class: 'quick-slider-icon',
        y_align: Clutter.ActorAlign.CENTER,
    });

    let slider = new Slider.Slider(value / 100.0);
    slider.x_expand = true;

    let label = new St.Label({
        text: `${value}%`,
        y_align: Clutter.ActorAlign.CENTER,
        style: 'min-width: 2.8em; text-align: right;',
    });

    let timeout = 0;
    slider.connect('notify::value', () => {
        let val = Math.round(slider.value * 100);
        label.text = `${val}%`;
        if (timeout) GLib.source_remove(timeout);
        timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
            onChange(val);
            timeout = 0;
            return GLib.SOURCE_REMOVE;
        });
    });

    row.add_child(icon);
    row.add_child(slider);
    row.add_child(label);
    return row;
}

export default class AcerMonitorExtension extends Extension {
    enable() {
        let state = getInitialState();
        let qs = Main.panel.statusArea.quickSettings;
        let menuBox = qs.menu.box;

        this._widgets = [];

        // === Container for sliders — inserted BEFORE the grid (index 0 in menuBox) ===
        let sliderContainer = new St.BoxLayout({
            vertical: true,
            x_expand: true,
            style: 'margin: 0; padding: 0;',
        });

        // Brightness
        sliderContainer.add_child(makeSliderRow(
            'display-brightness-symbolic',
            state.brightness,
            (val) => execCli(`brightness ${val}`)
        ));

        // Contrast
        sliderContainer.add_child(makeSliderRow(
            'display-symbolic',
            state.contrast,
            (val) => execCli(`contrast ${val}`)
        ));

        // Insert BEFORE everything else in the menu box (above the grid)
        menuBox.insert_child_at_index(sliderContainer, 0);
        this._widgets.push(sliderContainer);

        // === Preset SubMenu — added to sysMenu at position 0 ===
        // (appears as a pill below the grid, similar to other popup items)
        let presetSubMenu = new PopupMenu.PopupSubMenuMenuItem(
            `Preset: ${state.mode_name}`, true
        );
        presetSubMenu.icon.icon_name = 'video-display-symbolic';

        const MODES = [
            { label: 'User Mode',       short: 'User',     cmd: 'preset user' },
            { label: 'Standard Mode',   short: 'Standard', cmd: 'preset standard' },
            { label: 'ECO Power Saver', short: 'ECO',      cmd: 'preset eco' },
            { label: 'Graphics Mode',   short: 'Graphics', cmd: 'preset graphics' },
            { label: 'HDR Mode',        short: 'HDR',      cmd: 'preset hdr' },
            { label: 'Action Gaming',   short: 'Action',   cmd: 'preset action' },
            { label: 'Racing Mode',     short: 'Racing',   cmd: 'preset racing' },
            { label: 'Sports Mode',     short: 'Sports',   cmd: 'preset sports' },
        ];

        for (let m of MODES) {
            let item = new PopupMenu.PopupMenuItem(m.label);
            item.connect('activate', () => {
                execCli(m.cmd);
                presetSubMenu.label.text = `Preset: ${m.short}`;
            });
            presetSubMenu.menu.addMenuItem(item);
        }

        qs.menu.addMenuItem(presetSubMenu, 0);
        this._widgets.push(presetSubMenu);
    }

    disable() {
        for (let w of this._widgets ?? []) {
            w.destroy();
        }
        this._widgets = [];
    }
}
