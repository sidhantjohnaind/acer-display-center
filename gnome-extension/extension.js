import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as QuickSettings from 'resource:///org/gnome/shell/ui/quickSettings.js';
import * as Slider from 'resource:///org/gnome/shell/ui/slider.js';
import St from 'gi://St';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';

function execCli(cmd) {
    GLib.spawn_command_line_async(`/usr/local/bin/amctl ${cmd}`);
}

const PRESET_NAMES = {
    0: 'User',
    1: 'Standard',
    2: 'ECO',
    3: 'Graphics',
    4: 'HDR',
    5: 'Action',
    6: 'Racing',
    7: 'Sports',
    11: 'HDR'
};

function getInitialState() {
    try {
        let [res, stdout] = GLib.spawn_command_line_sync('/usr/local/bin/amctl info --json');
        if (res && stdout) {
            let str = new TextDecoder().decode(stdout);
            let parsed = JSON.parse(str);
            if (parsed && parsed.current_values) {
                let modeVal = parsed.current_values.display_mode ? parsed.current_values.display_mode.current : 1;
                return {
                    brightness: parsed.current_values.brightness ? parsed.current_values.brightness.current : 80,
                    contrast: parsed.current_values.contrast ? parsed.current_values.contrast.current : 50,
                    volume: parsed.current_values.volume ? parsed.current_values.volume.current : 100,
                    display_mode: modeVal,
                    mode_name: PRESET_NAMES[modeVal] || `Mode ${modeVal}`
                };
            }
        }
    } catch (e) {
        console.error(`AcerMonitor state error: ${e}`);
    }
    return { brightness: 80, contrast: 50, volume: 100, display_mode: 1, mode_name: 'Standard' };
}

export default class AcerMonitorExtension extends Extension {
    enable() {
        let state = getInitialState();
        let sysMenu = Main.panel.statusArea.quickSettings.menu;

        this._menuItems = [];

        // Presets SubMenu inserted at top (position 0)
        let presetSubMenu = new PopupMenu.PopupSubMenuMenuItem(`Preset: ${state.mode_name}`, true);
        presetSubMenu.style_class = 'quick-menu-toggle popup-menu-item';
        presetSubMenu.icon.icon_name = 'video-display-symbolic';
        sysMenu.addMenuItem(presetSubMenu, 0);
        this._menuItems.push(presetSubMenu);

        let modes = [
            { label: 'User Mode', shortName: 'User', cmd: 'preset user' },
            { label: 'Standard Mode', shortName: 'Standard', cmd: 'preset standard' },
            { label: 'ECO Power Saver', shortName: 'ECO', cmd: 'preset eco' },
            { label: 'Graphics Mode', shortName: 'Graphics', cmd: 'preset graphics' },
            { label: 'HDR Mode', shortName: 'HDR', cmd: 'preset hdr' },
            { label: 'Action Gaming', shortName: 'Action', cmd: 'preset action' },
            { label: 'Racing Mode', shortName: 'Racing', cmd: 'preset racing' },
            { label: 'Sports Mode', shortName: 'Sports', cmd: 'preset sports' },
        ];

        for (let m of modes) {
            let item = new PopupMenu.PopupMenuItem(m.label);
            item.connect('activate', () => {
                execCli(m.cmd);
                presetSubMenu.label.set_text(`Preset: ${m.shortName}`);
            });
            presetSubMenu.menu.addMenuItem(item);
        }

        // === Contrast Slider at position 0 ===
        let contrastItem = new PopupMenu.PopupBaseMenuItem({ style_class: 'quick-slider', reactive: true, activate: false });
        let contrastIcon = new St.Icon({
            icon_name: 'display-symbolic',
            style_class: 'quick-slider-icon',
        });
        let contrastSlider = new Slider.Slider(state.contrast / 100.0);
        let contrastLabel = new St.Label({
            text: `${state.contrast}%`,
            y_align: 1,
            style: 'min-width: 2.5em; text-align: right;'
        });

        let contrastTimeout = 0;
        contrastSlider.connect('notify::value', () => {
            let val = Math.round(contrastSlider.value * 100);
            contrastLabel.text = `${val}%`;
            if (contrastTimeout) GLib.source_remove(contrastTimeout);
            contrastTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`contrast ${val}`);
                contrastTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });

        contrastItem.add_child(contrastIcon);
        contrastItem.add_child(contrastSlider);
        contrastItem.add_child(contrastLabel);
        sysMenu.addMenuItem(contrastItem, 0);
        this._menuItems.push(contrastItem);

        // === Brightness Slider at position 0 (ends up first) ===
        let brightItem = new PopupMenu.PopupBaseMenuItem({ style_class: 'quick-slider', reactive: true, activate: false });
        let brightIcon = new St.Icon({
            icon_name: 'display-brightness-symbolic',
            style_class: 'quick-slider-icon',
        });
        let brightSlider = new Slider.Slider(state.brightness / 100.0);
        let brightLabel = new St.Label({
            text: `${state.brightness}%`,
            y_align: 1,
            style: 'min-width: 2.5em; text-align: right;'
        });

        let brightTimeout = 0;
        brightSlider.connect('notify::value', () => {
            let val = Math.round(brightSlider.value * 100);
            brightLabel.text = `${val}%`;
            if (brightTimeout) GLib.source_remove(brightTimeout);
            brightTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`brightness ${val}`);
                brightTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });

        brightItem.add_child(brightIcon);
        brightItem.add_child(brightSlider);
        brightItem.add_child(brightLabel);
        sysMenu.addMenuItem(brightItem, 0);
        this._menuItems.push(brightItem);

    }

    disable() {
        if (this._menuItems) {
            for (let item of this._menuItems) {
                item.destroy();
            }
            this._menuItems = [];
        }
    }
}
