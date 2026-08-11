import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as QuickSettings from 'resource:///org/gnome/shell/ui/quickSettings.js';
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
        console.error(`AcerMonitor QuickSettings state error: ${e}`);
    }
    return { brightness: 80, contrast: 50, volume: 100, display_mode: 1, mode_name: 'Standard' };
}

// Full-width Brightness Slider
const AcerBrightnessSlider = GObject.registerClass(
class AcerBrightnessSlider extends QuickSettings.QuickSlider {
    _init(initialVal) {
        super._init();
        this.iconName = 'display-brightness-symbolic';
        this.slider.value = initialVal / 100.0;

        let timeout = 0;
        this.slider.connect('notify::value', () => {
            let val = Math.round(this.slider.value * 100);
            if (timeout) GLib.source_remove(timeout);
            timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`brightness ${val}`);
                timeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }
});

// Full-width Contrast Slider
const AcerContrastSlider = GObject.registerClass(
class AcerContrastSlider extends QuickSettings.QuickSlider {
    _init(initialVal) {
        super._init();
        this.iconName = 'display-symbolic';
        this.slider.value = initialVal / 100.0;

        let timeout = 0;
        this.slider.connect('notify::value', () => {
            let val = Math.round(this.slider.value * 100);
            if (timeout) GLib.source_remove(timeout);
            timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`contrast ${val}`);
                timeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }
});

// Presets Menu Toggle Pill
const AcerPresetsToggle = GObject.registerClass(
class AcerPresetsToggle extends QuickSettings.QuickMenuToggle {
    _init(initialModeName) {
        super._init({
            title: `Preset: ${initialModeName}`,
            iconName: 'video-display-symbolic',
            toggleMode: false,
        });

        this.menu.setHeader('video-display-symbolic', 'Presets');

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
            this.menu.addAction(m.label, () => {
                execCli(m.cmd);
                this.title = `Preset: ${m.shortName}`;
            });
        }
    }
});

export default class AcerMonitorExtension extends Extension {
    enable() {
        let state = getInitialState();

        this._brightSlider = new AcerBrightnessSlider(state.brightness);
        this._contrastSlider = new AcerContrastSlider(state.contrast);
        this._presetsToggle = new AcerPresetsToggle(state.mode_name);

        let grid = Main.panel.statusArea.quickSettings._grid;

        // Insert sliders at the very top (indices 0 & 1) spanning 2 columns (full width)!
        grid.insert_child_at_index(this._brightSlider, 0);
        grid.insert_child_at_index(this._contrastSlider, 1);

        try {
            grid.set_child_packing(this._brightSlider, true, true, 2, 0);
            grid.set_child_packing(this._contrastSlider, true, true, 2, 0);
        } catch (e) {
            // Fallback for GNOME layout packing
        }

        // Add Presets pill button to the grid
        Main.panel.statusArea.quickSettings.addItem(this._presetsToggle);
    }

    disable() {
        if (this._brightSlider) { this._brightSlider.destroy(); this._brightSlider = null; }
        if (this._contrastSlider) { this._contrastSlider.destroy(); this._contrastSlider = null; }
        if (this._presetsToggle) { this._presetsToggle.destroy(); this._presetsToggle = null; }
    }
}
