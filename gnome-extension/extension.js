import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Slider from 'resource:///org/gnome/shell/ui/slider.js';
import GLib from 'gi://GLib';

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

        // Separator
        let sep1 = new PopupMenu.PopupSeparatorMenuItem();
        sysMenu.addMenuItem(sep1);
        this._menuItems.push(sep1);

        // Header
        let headerItem = new PopupMenu.PopupMenuItem('🖥️ Acer Monitor Control', { reactive: false });
        sysMenu.addMenuItem(headerItem);
        this._menuItems.push(headerItem);

        // Brightness Slider
        let brightTimeout = 0;
        let brightLabel = new PopupMenu.PopupMenuItem(`  Brightness (${state.brightness}%)`, { reactive: false });
        sysMenu.addMenuItem(brightLabel);
        this._menuItems.push(brightLabel);

        let brightSliderItem = new PopupMenu.PopupBaseMenuItem({ reactive: true, activate: false });
        let brightSlider = new Slider.Slider(state.brightness / 100.0);
        brightSlider.connect('notify::value', () => {
            let val = Math.round(brightSlider.value * 100);
            brightLabel.label.set_text(`  Brightness (${val}%)`);
            if (brightTimeout) GLib.source_remove(brightTimeout);
            brightTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`brightness ${val}`);
                brightTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
        brightSliderItem.add_child(brightSlider);
        sysMenu.addMenuItem(brightSliderItem);
        this._menuItems.push(brightSliderItem);

        // Contrast Slider
        let contrastTimeout = 0;
        let contrastLabel = new PopupMenu.PopupMenuItem(`  Contrast (${state.contrast}%)`, { reactive: false });
        sysMenu.addMenuItem(contrastLabel);
        this._menuItems.push(contrastLabel);

        let contrastSliderItem = new PopupMenu.PopupBaseMenuItem({ reactive: true, activate: false });
        let contrastSlider = new Slider.Slider(state.contrast / 100.0);
        contrastSlider.connect('notify::value', () => {
            let val = Math.round(contrastSlider.value * 100);
            contrastLabel.label.set_text(`  Contrast (${val}%)`);
            if (contrastTimeout) GLib.source_remove(contrastTimeout);
            contrastTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`contrast ${val}`);
                contrastTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
        contrastSliderItem.add_child(contrastSlider);
        sysMenu.addMenuItem(contrastSliderItem);
        this._menuItems.push(contrastSliderItem);

        // Presets Header & Submenu Items
        let presetHeader = new PopupMenu.PopupMenuItem(`  Presets (Active: ${state.mode_name})`, { reactive: false });
        sysMenu.addMenuItem(presetHeader);
        this._menuItems.push(presetHeader);

        let modes = [
            { label: '    User Mode', shortName: 'User', cmd: 'preset user' },
            { label: '    Standard Mode', shortName: 'Standard', cmd: 'preset standard' },
            { label: '    ECO Power Saver', shortName: 'ECO', cmd: 'preset eco' },
            { label: '    Graphics Mode', shortName: 'Graphics', cmd: 'preset graphics' },
            { label: '    HDR Mode', shortName: 'HDR', cmd: 'preset hdr' },
            { label: '    Action Gaming', shortName: 'Action', cmd: 'preset action' },
            { label: '    Racing Mode', shortName: 'Racing', cmd: 'preset racing' },
            { label: '    Sports Mode', shortName: 'Sports', cmd: 'preset sports' },
        ];

        for (let m of modes) {
            let item = new PopupMenu.PopupMenuItem(m.label);
            item.connect('activate', () => {
                execCli(m.cmd);
                presetHeader.label.set_text(`  Presets (Active: ${m.shortName})`);
            });
            sysMenu.addMenuItem(item);
            this._menuItems.push(item);
        }
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
