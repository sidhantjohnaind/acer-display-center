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
/*  Helper Functions                                                  */
/* ------------------------------------------------------------------ */
function findAmctl() {
    for (const path of ['/usr/local/bin/amctl', '/usr/bin/amctl', '/usr/local/bin/acer_monitor_cli']) {
        if (GLib.file_test(path, GLib.FileTest.EXISTS)) return path;
    }
    return 'amctl';
}

function execCli(cmd) {
    const amctl = findAmctl();
    try {
        GLib.spawn_command_line_async(`${amctl} ${cmd}`);
    } catch (e) {
        log(`[AcerMonitor] Error executing command: ${cmd} - ${e.message}`);
    }
}

function getInitialState() {
    const amctl = findAmctl();
    try {
        let [res, stdout] = GLib.spawn_command_line_sync(`${amctl} info --json`);
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

                let inputVal = cv.input?.current ?? 0x0F;
                let inputName = cv.input?.name ?? ({
                    0x01: 'Auto', 0x0F: 'DisplayPort', 0x11: 'HDMI 1', 0x12: 'HDMI 2'
                }[inputVal] ?? 'DisplayPort');

                return {
                    brightness: cv.brightness?.current ?? 80,
                    contrast:   cv.contrast?.current   ?? 50,
                    volume:     cv.volume?.current     ?? 50,
                    mute:       cv.mute?.current === 1,
                    modeVal,
                    modeName,
                    inputVal,
                    inputName,
                    blackBoost: cv.black_boost?.current ?? 5,
                };
            }
        }
    } catch (_) {}
    return {
        brightness: 80,
        contrast: 50,
        volume: 50,
        mute: false,
        modeVal: 1,
        modeName: 'Standard',
        inputVal: 0x0F,
        inputName: 'DisplayPort',
        blackBoost: 5,
    };
}

/* ------------------------------------------------------------------ */
/*  Brightness Slider                                                 */
/* ------------------------------------------------------------------ */
const AcerBrightnessSlider = GObject.registerClass(
class AcerBrightnessSlider extends QuickSlider {
    _init(initialValue) {
        super._init({
            iconName: 'display-brightness-symbolic',
        });
        this.menuEnabled = false;
        this.slider.value = initialValue / 100.0;
        this._updateIcon(initialValue);

        this._timeout = 0;
        this.slider.connect('notify::value', () => {
            const val = Math.round(this.slider.value * 100);
            this._updateIcon(val);
            if (this._timeout) GLib.source_remove(this._timeout);
            this._timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`brightness ${val}`);
                this._timeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }

    _updateIcon(val) {
        if (val > 66) {
            this.iconName = 'display-brightness-high-symbolic';
        } else if (val > 33) {
            this.iconName = 'display-brightness-medium-symbolic';
        } else {
            this.iconName = 'display-brightness-low-symbolic';
        }
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
/*  Contrast Slider                                                   */
/* ------------------------------------------------------------------ */
const AcerContrastSlider = GObject.registerClass(
class AcerContrastSlider extends QuickSlider {
    _init(initialValue) {
        super._init({
            iconName: 'display-symbolic',
        });
        this.menuEnabled = false;
        this.slider.value = initialValue / 100.0;

        this._timeout = 0;
        this.slider.connect('notify::value', () => {
            const val = Math.round(this.slider.value * 100);
            if (this._timeout) GLib.source_remove(this._timeout);
            this._timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`contrast ${val}`);
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
/*  Volume Slider                                                     */
/* ------------------------------------------------------------------ */
const AcerVolumeSlider = GObject.registerClass(
class AcerVolumeSlider extends QuickSlider {
    _init(initialVolume, isMuted) {
        super._init({
            iconName: isMuted ? 'audio-volume-muted-symbolic' : 'audio-volume-high-symbolic',
        });
        this.menuEnabled = false;
        this.slider.value = initialVolume / 100.0;

        this._timeout = 0;
        this.slider.connect('notify::value', () => {
            const val = Math.round(this.slider.value * 100);
            if (this._timeout) GLib.source_remove(this._timeout);
            this._timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`volume ${val}`);
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
/*  Preset Toggle Pill & Submenu                                       */
/* ------------------------------------------------------------------ */
const PRESETS = [
    { label: 'Standard Mode',   short: 'Standard', cmd: 'preset standard', key: 'Standard' },
    { label: 'ECO Power Saver', short: 'ECO',      cmd: 'preset eco',      key: 'ECO' },
    { label: 'HDR Game Mode',   short: 'HDR',      cmd: 'preset hdr',      key: 'HDR' },
    { label: 'Action Gaming',   short: 'Action',   cmd: 'preset action',   key: 'Action' },
    { label: 'Racing Mode',     short: 'Racing',   cmd: 'preset racing',   key: 'Racing' },
    { label: 'Sports Mode',     short: 'Sports',   cmd: 'preset sports',   key: 'Sports' },
    { label: 'Graphics Mode',   short: 'Graphics', cmd: 'preset graphics', key: 'Graphics' },
    { label: 'Reading / Text',  short: 'Reading',  cmd: 'preset reading',  key: 'Reading' },
    { label: 'Movie / Cinema',  short: 'Movie',    cmd: 'preset movie',    key: 'Movie' },
    { label: 'User Mode',       short: 'User',     cmd: 'preset user',     key: 'User' },
];

const AcerPresetToggle = GObject.registerClass(
class AcerPresetToggle extends QuickMenuToggle {
    _init(initialModeName) {
        super._init({
            title: 'Display Mode',
            subtitle: initialModeName,
            iconName: 'video-display-symbolic',
        });

        this.menu.setHeader('video-display-symbolic', 'Acer Display Modes', 'One-touch hardware OSD presets');

        const section = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(section);

        this._itemsMap = new Map();
        for (const p of PRESETS) {
            const item = new PopupMenu.PopupMenuItem(p.label);
            item.connect('activate', () => {
                execCli(p.cmd);
                this.subtitle = p.short;
                this._updateActiveOrnament(p.key);
            });
            section.addMenuItem(item);
            this._itemsMap.set(p.key, item);
        }

        this._updateActiveOrnament(initialModeName);
    }

    _updateActiveOrnament(activeKey) {
        for (const [key, item] of this._itemsMap.entries()) {
            if (key === activeKey || activeKey.includes(key)) {
                item.setOrnament(PopupMenu.Ornament.CHECK);
            } else {
                item.setOrnament(PopupMenu.Ornament.NONE);
            }
        }
    }
});

/* ------------------------------------------------------------------ */
/*  Input Source Toggle Pill & Submenu                                 */
/* ------------------------------------------------------------------ */
const INPUTS = [
    { label: 'DisplayPort', short: 'DisplayPort', cmd: 'input dp',    key: 'DisplayPort' },
    { label: 'HDMI 1',      short: 'HDMI 1',      cmd: 'input hdmi1', key: 'HDMI 1' },
    { label: 'HDMI 2',      short: 'HDMI 2',      cmd: 'input hdmi2', key: 'HDMI 2' },
    { label: 'Auto-Switch', short: 'Auto',        cmd: 'input auto',  key: 'Auto' },
];

const AcerInputToggle = GObject.registerClass(
class AcerInputToggle extends QuickMenuToggle {
    _init(initialInputName) {
        super._init({
            title: 'Input Source',
            subtitle: initialInputName,
            iconName: 'display-symbolic',
        });

        this.menu.setHeader('display-symbolic', 'Monitor Input Source', 'Select active display signal source');

        const section = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(section);

        this._itemsMap = new Map();
        for (const inp of INPUTS) {
            const item = new PopupMenu.PopupMenuItem(inp.label);
            item.connect('activate', () => {
                execCli(inp.cmd);
                this.subtitle = inp.short;
                this._updateActiveOrnament(inp.key);
            });
            section.addMenuItem(item);
            this._itemsMap.set(inp.key, item);
        }

        this._updateActiveOrnament(initialInputName);
    }

    _updateActiveOrnament(activeKey) {
        for (const [key, item] of this._itemsMap.entries()) {
            if (key === activeKey || activeKey.includes(key)) {
                item.setOrnament(PopupMenu.Ornament.CHECK);
            } else {
                item.setOrnament(PopupMenu.Ornament.NONE);
            }
        }
    }
});

/* ------------------------------------------------------------------ */
/*  Gaming & Enhancements Submenu Pill                                */
/* ------------------------------------------------------------------ */
const AcerGamingToggle = GObject.registerClass(
class AcerGamingToggle extends QuickMenuToggle {
    _init(initialBlackBoost) {
        super._init({
            title: 'Gaming & Vision',
            subtitle: `Boost: ${initialBlackBoost}`,
            iconName: 'preferences-desktop-display-symbolic',
        });

        this.menu.setHeader('preferences-desktop-display-symbolic', 'Gaming & Hardware Enhancements', 'Black Boost, OverDrive, AimPoint & Solar');

        const section = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(section);

        // Section: Black Boost
        section.addMenuItem(new PopupMenu.PopupSeparatorMenuItem('Black Boost'));
        for (const lvl of [0, 2, 5, 8, 10]) {
            const item = new PopupMenu.PopupMenuItem(`Black Boost Level ${lvl}`);
            item.connect('activate', () => {
                execCli(`blackboost ${lvl}`);
                this.subtitle = `Boost: ${lvl}`;
            });
            section.addMenuItem(item);
        }

        // Section: Blue Light Filter
        section.addMenuItem(new PopupMenu.PopupSeparatorMenuItem('Blue Light Filter'));
        const bluelightLevels = [
            { label: 'Off / Standard (0%)', cmd: 'bluelight 0' },
            { label: 'Level 1 (50%)',       cmd: 'bluelight 50' },
            { label: 'Level 2 (60%)',       cmd: 'bluelight 60' },
            { label: 'Level 3 (70%)',       cmd: 'bluelight 70' },
            { label: 'Level 4 (80%)',       cmd: 'bluelight 80' },
        ];
        for (const bl of bluelightLevels) {
            const item = new PopupMenu.PopupMenuItem(bl.label);
            item.connect('activate', () => execCli(bl.cmd));
            section.addMenuItem(item);
        }

        // Section: OverDrive & AimPoint
        section.addMenuItem(new PopupMenu.PopupSeparatorMenuItem('Hardware OverDrive & AimPoint'));
        const odExtreme = new PopupMenu.PopupMenuItem('OverDrive: Extreme (2)');
        odExtreme.connect('activate', () => execCli('od 2'));
        section.addMenuItem(odExtreme);

        const odNormal = new PopupMenu.PopupMenuItem('OverDrive: Normal (1)');
        odNormal.connect('activate', () => execCli('od 1'));
        section.addMenuItem(odNormal);

        const aimToggle = new PopupMenu.PopupMenuItem('AimPoint Crosshair: Cycle');
        aimToggle.connect('activate', () => execCli('aim 1'));
        section.addMenuItem(aimToggle);

        // Section: Solar Circadian Schedule
        section.addMenuItem(new PopupMenu.PopupSeparatorMenuItem('Solar Schedule & Utilities'));
        const dayMode = new PopupMenu.PopupMenuItem('☀️ Apply Day Mode (Brightness 90%)');
        dayMode.connect('activate', () => execCli('brightness 90 --osd'));
        section.addMenuItem(dayMode);

        const nightMode = new PopupMenu.PopupMenuItem('🌙 Apply Night Mode (Brightness 20% + Warm)');
        nightMode.connect('activate', () => {
            execCli('brightness 20 --osd');
            execCli('colortemp warm');
        });
        section.addMenuItem(nightMode);

        const unlockItem = new PopupMenu.PopupMenuItem('🔓 Emergency Unlock OSD Keys');
        unlockItem.connect('activate', () => execCli('unlock'));
        section.addMenuItem(unlockItem);
    }
});

/* ------------------------------------------------------------------ */
/*  Main Extension Entry Point                                        */
/* ------------------------------------------------------------------ */
export default class AcerMonitorExtension extends Extension {
    enable() {
        const state = getInitialState();
        const sysMenu = Main.panel.statusArea.quickSettings.menu;

        this._items = [];

        // Create quick control items
        const bright   = new AcerBrightnessSlider(state.brightness);
        const contrast = new AcerContrastSlider(state.contrast);
        const volume   = new AcerVolumeSlider(state.volume, state.mute);

        const preset   = new AcerPresetToggle(state.modeName);
        const input    = new AcerInputToggle(state.inputName);
        const gaming   = new AcerGamingToggle(state.blackBoost);

        // Add sliders & pills to QuickSettings grid
        sysMenu.addItem(bright, 2);
        sysMenu.addItem(contrast, 1);
        sysMenu.addItem(volume, 1);

        sysMenu.addItem(preset, 1);
        sysMenu.addItem(input, 1);
        sysMenu.addItem(gaming, 1);

        this._items.push(bright, contrast, volume, preset, input, gaming);
    }

    disable() {
        for (const w of this._items ?? []) {
            try { w.destroy(); } catch (_) {}
        }
        this._items = [];
    }
}

