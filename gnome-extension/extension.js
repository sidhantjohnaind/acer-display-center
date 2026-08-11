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

const AcerMonitorIndicator = GObject.registerClass(
class AcerMonitorIndicator extends QuickSettings.SystemIndicator {
    _init() {
        super._init();

        let state = getInitialState();

        // Top right system tray icon
        this._icon = this._addIndicator();
        this._icon.icon_name = 'display-brightness-symbolic';

        // Full-width Brightness Slider
        this._brightSlider = new QuickSettings.QuickSlider();
        this._brightSlider.iconName = 'display-brightness-symbolic';
        this._brightSlider.slider.value = state.brightness / 100.0;

        let brightTimeout = 0;
        this._brightSlider.slider.connect('notify::value', () => {
            let val = Math.round(this._brightSlider.slider.value * 100);
            if (brightTimeout) GLib.source_remove(brightTimeout);
            brightTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`brightness ${val}`);
                brightTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });

        // Full-width Contrast Slider
        this._contrastSlider = new QuickSettings.QuickSlider();
        this._contrastSlider.iconName = 'display-symbolic';
        this._contrastSlider.slider.value = state.contrast / 100.0;

        let contrastTimeout = 0;
        this._contrastSlider.slider.connect('notify::value', () => {
            let val = Math.round(this._contrastSlider.slider.value * 100);
            if (contrastTimeout) GLib.source_remove(contrastTimeout);
            contrastTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`contrast ${val}`);
                contrastTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });

        // Presets Toggle Pill
        this._presetsToggle = new QuickSettings.QuickMenuToggle({
            title: `Preset: ${state.mode_name}`,
            iconName: 'video-display-symbolic',
            toggleMode: false,
        });
        this._presetsToggle.menu.setHeader('video-display-symbolic', 'Presets');

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
            this._presetsToggle.menu.addAction(m.label, () => {
                execCli(m.cmd);
                this._presetsToggle.title = `Preset: ${m.shortName}`;
            });
        }

        // Attach quickSettingsItems array to SystemIndicator
        this.quickSettingsItems = [this._brightSlider, this._contrastSlider, this._presetsToggle];
    }
});

export default class AcerMonitorExtension extends Extension {
    enable() {
        this._idleId = GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
            if (Main.panel.statusArea.quickSettings) {
                this._indicator = new AcerMonitorIndicator();
                Main.panel.statusArea.quickSettings.addExternalIndicator(this._indicator);
            }
            this._idleId = 0;
            return GLib.SOURCE_REMOVE;
        });
    }

    disable() {
        if (this._idleId) {
            GLib.source_remove(this._idleId);
            this._idleId = 0;
        }
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
    }
}
