import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Slider from 'resource:///org/gnome/shell/ui/slider.js';
import St from 'gi://St';
import GLib from 'gi://GLib';
import Clutter from 'gi://Clutter';
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

const AcerMonitorIndicator = GObject.registerClass(
class AcerMonitorIndicator extends PanelMenu.Button {
    _init() {
        super._init(0.0, 'Acer Monitor Control', false);

        let state = getInitialState();

        let icon = new St.Icon({
            icon_name: 'display-symbolic',
            style_class: 'system-status-icon',
        });
        this.add_child(icon);

        this.connect('scroll-event', (actor, event) => {
            let direction = event.get_scroll_direction();
            if (direction === Clutter.ScrollDirection.UP) {
                execCli('brightness +5 --osd');
            } else if (direction === Clutter.ScrollDirection.DOWN) {
                execCli('brightness -5 --osd');
            }
            return Clutter.EVENT_STOP;
        });

        let titleItem = new PopupMenu.PopupMenuItem('🖥️ Acer Monitor Control', { reactive: false });
        this.menu.addMenuItem(titleItem);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        // Brightness Section
        let brightTimeout = 0;
        let brightLabelItem = new PopupMenu.PopupMenuItem(`Brightness (${state.brightness}%)`, { reactive: false });
        this.menu.addMenuItem(brightLabelItem);

        let brightSliderItem = new PopupMenu.PopupBaseMenuItem({ reactive: true, activate: false });
        let brightSlider = new Slider.Slider(state.brightness / 100.0);

        let applyBrightness = () => {
            let val = Math.round(brightSlider.value * 100);
            brightLabelItem.label.set_text(`Brightness (${val}%)`);
            execCli(`brightness ${val}`);
        };

        brightSlider.connect('drag-end', applyBrightness);
        brightSlider.connect('notify::value', () => {
            let val = Math.round(brightSlider.value * 100);
            brightLabelItem.label.set_text(`Brightness (${val}%)`);
            if (brightTimeout) {
                GLib.source_remove(brightTimeout);
            }
            brightTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`brightness ${val}`);
                brightTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
        brightSliderItem.add_child(brightSlider);
        this.menu.addMenuItem(brightSliderItem);

        // Contrast Section
        let contrastTimeout = 0;
        let contrastLabelItem = new PopupMenu.PopupMenuItem(`Contrast (${state.contrast}%)`, { reactive: false });
        this.menu.addMenuItem(contrastLabelItem);

        let contrastSliderItem = new PopupMenu.PopupBaseMenuItem({ reactive: true, activate: false });
        let contrastSlider = new Slider.Slider(state.contrast / 100.0);

        let applyContrast = () => {
            let val = Math.round(contrastSlider.value * 100);
            contrastLabelItem.label.set_text(`Contrast (${val}%)`);
            execCli(`contrast ${val}`);
        };

        contrastSlider.connect('drag-end', applyContrast);
        contrastSlider.connect('notify::value', () => {
            let val = Math.round(contrastSlider.value * 100);
            contrastLabelItem.label.set_text(`Contrast (${val}%)`);
            if (contrastTimeout) {
                GLib.source_remove(contrastTimeout);
            }
            contrastTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`contrast ${val}`);
                contrastTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
        contrastSliderItem.add_child(contrastSlider);
        this.menu.addMenuItem(contrastSliderItem);

        // Volume Section
        let volumeTimeout = 0;
        let volumeLabelItem = new PopupMenu.PopupMenuItem(`Volume (${state.volume}%)`, { reactive: false });
        this.menu.addMenuItem(volumeLabelItem);

        let volumeSliderItem = new PopupMenu.PopupBaseMenuItem({ reactive: true, activate: false });
        let volumeSlider = new Slider.Slider(state.volume / 100.0);

        let applyVolume = () => {
            let val = Math.round(volumeSlider.value * 100);
            volumeLabelItem.label.set_text(`Volume (${val}%)`);
            execCli(`volume ${val}`);
        };

        volumeSlider.connect('drag-end', applyVolume);
        volumeSlider.connect('notify::value', () => {
            let val = Math.round(volumeSlider.value * 100);
            volumeLabelItem.label.set_text(`Volume (${val}%)`);
            if (volumeTimeout) {
                GLib.source_remove(volumeTimeout);
            }
            volumeTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`volume ${val}`);
                volumeTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
        volumeSliderItem.add_child(volumeSlider);
        this.menu.addMenuItem(volumeSliderItem);

        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        // Presets Section displaying active preset
        let presetHeader = new PopupMenu.PopupMenuItem(`Presets (Active: ${state.mode_name})`, { reactive: false });
        this.menu.addMenuItem(presetHeader);

        let modes = [
            { label: '  User Mode', shortName: 'User', cmd: 'preset user' },
            { label: '  Standard Mode', shortName: 'Standard', cmd: 'preset standard' },
            { label: '  ECO Power Saver', shortName: 'ECO', cmd: 'preset eco' },
            { label: '  Graphics Mode', shortName: 'Graphics', cmd: 'preset graphics' },
            { label: '  HDR Mode', shortName: 'HDR', cmd: 'preset hdr' },
            { label: '  Action Gaming', shortName: 'Action', cmd: 'preset action' },
            { label: '  Racing Mode', shortName: 'Racing', cmd: 'preset racing' },
            { label: '  Sports Mode', shortName: 'Sports', cmd: 'preset sports' },
        ];

        for (let m of modes) {
            let item = new PopupMenu.PopupMenuItem(m.label);
            item.connect('activate', () => {
                execCli(m.cmd);
                presetHeader.label.set_text(`Presets (Active: ${m.shortName})`);
            });
            this.menu.addMenuItem(item);
        }
    }
});

export default class AcerMonitorExtension extends Extension {
    enable() {
        this._indicator = new AcerMonitorIndicator();
        Main.panel.addToStatusArea(this.uuid, this._indicator);
    }

    disable() {
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
    }
}
