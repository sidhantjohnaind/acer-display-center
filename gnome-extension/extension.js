import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as QuickSettings from 'resource:///org/gnome/shell/ui/quickSettings.js';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';

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
                return {
                    brightness: parsed.current_values.brightness ? parsed.current_values.brightness.current : 80,
                    contrast: parsed.current_values.contrast ? parsed.current_values.contrast.current : 50,
                    volume: parsed.current_values.volume ? parsed.current_values.volume.current : 100
                };
            }
        }
    } catch (e) {
        console.error(`AcerMonitor QuickSettings state error: ${e}`);
    }
    return { brightness: 80, contrast: 50, volume: 100 };
}

const AcerMonitorIndicator = GObject.registerClass(
class AcerMonitorIndicator extends QuickSettings.SystemIndicator {
    _init() {
        super._init();

        let state = getInitialState();

        // Top right system indicator icon
        this._icon = this._addIndicator();
        this._icon.icon_name = 'display-brightness-symbolic';

        // Brightness Slider in System Nav
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

        // Contrast Slider in System Nav
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

        // Presets Toggle Pill in System Nav
        this._presetsToggle = new QuickSettings.QuickMenuToggle({
            title: 'Presets',
            iconName: 'video-display-symbolic',
            toggleMode: false,
        });
        this._presetsToggle.menu.setHeader('video-display-symbolic', 'Presets');

        let modes = [
            { name: 'User Mode', cmd: 'preset user' },
            { name: 'Standard Mode', cmd: 'preset standard' },
            { name: 'ECO Power Saver', cmd: 'preset eco' },
            { name: 'Graphics Mode', cmd: 'preset graphics' },
            { name: 'HDR Mode', cmd: 'preset hdr' },
            { name: 'Action Gaming', cmd: 'preset action' },
            { name: 'Racing Mode', cmd: 'preset racing' },
            { name: 'Sports Mode', cmd: 'preset sports' },
        ];

        for (let m of modes) {
            this._presetsToggle.menu.addAction(m.name, () => execCli(m.cmd));
        }

        // Attach all controls into System QuickSettings Nav
        this.quickSettingsItems = [this._brightSlider, this._contrastSlider, this._presetsToggle];
    }
});

export default class AcerMonitorExtension extends Extension {
    enable() {
        this._indicator = new AcerMonitorIndicator();
        Main.panel.statusArea.quickSettings.addExternalIndicator(this._indicator);
    }

    disable() {
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
    }
}
