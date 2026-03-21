// frontend/modules/style.js
import { state } from './state.js';
import { rgbToHex, hexToRgb, appendLog, debounce } from './utils.js';

const { invoke } = window.__TAURI__ ? window.__TAURI__.core : { invoke: () => {} };

export async function loadStyle() {
    const colorBg = document.getElementById('color-bg');
    const colorText = document.getElementById('color-text');
    const colorBtnBg = document.getElementById('color-btn-bg');
    const colorBtnText = document.getElementById('color-btn-text');
    const fontSize = document.getElementById('font-size');
    const chkBtnRounding = document.getElementById('chk-btn-rounding');
    const btnRoundingValue = document.getElementById('btn-rounding-value');
    const chkPulse = document.getElementById('chk-pulse');
    const pulseSpeed = document.getElementById('pulse-speed');

    try {
        const style = await invoke('get_style_config');
        state.currentStyle = style;

        const isDark = style.theme !== 'light';
        if (colorBg) colorBg.value = rgbToHex(isDark ? style.dark_bg : style.light_bg);
        if (colorText) colorText.value = rgbToHex(isDark ? style.dark_text : style.light_text);
        if (colorBtnBg) colorBtnBg.value = rgbToHex(isDark ? style.dark_btn_bg : style.light_btn_bg);
        if (colorBtnText) colorBtnText.value = rgbToHex(isDark ? style.dark_btn_text : style.light_btn_text);
        
        if (style.font_size && fontSize) {
            fontSize.value = style.font_size;
            document.documentElement.style.setProperty('--font-size', style.font_size + 'px');
        }
        if (chkBtnRounding) chkBtnRounding.checked = style.btn_rounding_enabled ?? true;
        if (btnRoundingValue) btnRoundingValue.value = style.btn_rounding_value ?? 4.0;
        if (chkPulse) chkPulse.checked = style.progress_pulse_enabled ?? true;
        if (pulseSpeed) pulseSpeed.value = style.progress_pulse_speed ?? 1.0;

        applyColors(style);
    } catch (e) {
        console.error(e);
    }
}

export async function saveStyle() {
    const chkBtnRounding = document.getElementById('chk-btn-rounding');
    const btnRoundingValue = document.getElementById('btn-rounding-value');
    const chkPulse = document.getElementById('chk-pulse');
    const pulseSpeed = document.getElementById('pulse-speed');
    const fsInput = document.getElementById('font-size');

    if (!state.currentStyle) return;
    if (chkBtnRounding) state.currentStyle.btn_rounding_enabled = chkBtnRounding.checked;
    if (btnRoundingValue) state.currentStyle.btn_rounding_value = parseFloat(btnRoundingValue.value) || 4.0;
    if (chkPulse) state.currentStyle.progress_pulse_enabled = chkPulse.checked;
    if (pulseSpeed) state.currentStyle.progress_pulse_speed = parseFloat(pulseSpeed.value) || 1.0;
    if (fsInput) state.currentStyle.font_size = parseFloat(fsInput.value) || 15;

    await invoke('save_style_config', { config: state.currentStyle });
    applyColors(state.currentStyle);
}

export function applyColors(style) {
    const progressBar = document.getElementById('progress-bar');
    const isDark = style.theme !== 'light';
    const bg = isDark ? style.dark_bg : style.light_bg;
    const txt = isDark ? style.dark_text : style.light_text;
    const btnBg = isDark ? style.dark_btn_bg : style.light_btn_bg;
    const btnTxt = isDark ? style.dark_btn_text : style.light_btn_text;
    const inputBg = isDark ? style.dark_input_bg : style.light_input_bg;
    const listBg = isDark ? style.dark_list_bg : style.light_list_bg;

    if (bg) document.documentElement.style.setProperty('--bg-color', `rgb(${bg[0]},${bg[1]},${bg[2]})`);
    if (txt) document.documentElement.style.setProperty('--text-color', `rgb(${txt[0]},${txt[1]},${txt[2]})`);
    if (btnBg) document.documentElement.style.setProperty('--btn-bg', `rgb(${btnBg[0]},${btnBg[1]},${btnBg[2]})`);
    if (btnTxt) document.documentElement.style.setProperty('--btn-text', `rgb(${btnTxt[0]},${btnTxt[1]},${btnTxt[2]})`);
    if (inputBg) document.documentElement.style.setProperty('--input-bg', `rgb(${inputBg[0]},${inputBg[1]},${inputBg[2]})`);
    if (listBg) document.documentElement.style.setProperty('--list-bg', `rgb(${listBg[0]},${listBg[1]},${listBg[2]})`);

    if (style.font_size) document.documentElement.style.setProperty('--font-size', `${style.font_size}px`);

    if (style.btn_rounding_enabled !== false) {
        document.documentElement.style.setProperty('--border-radius', `${style.btn_rounding_value ?? 4.0}px`);
    } else {
        document.documentElement.style.setProperty('--border-radius', '0px');
    }

    if (style.progress_pulse_enabled && progressBar) {
        const speed = style.progress_pulse_speed ?? 1.0;
        progressBar.style.animation = `pulse ${2.0 / Math.max(0.1, speed)}s infinite`;
    } else if (progressBar) {
        progressBar.style.animation = 'none';
    }

    if (style.instance_overrides) {
        for (const [id, override] of Object.entries(style.instance_overrides)) {
            const el = document.getElementById(id);
            if (el) {
                if (override.bg) el.style.backgroundColor = `rgb(${override.bg[0]},${override.bg[1]},${override.bg[2]})`;
                if (override.text) el.style.color = `rgb(${override.text[0]},${override.text[1]},${override.text[2]})`;
                if (override.rounding !== undefined) el.style.borderRadius = `${override.rounding}px`;
            }
        }
    }
}

export function updatePaletteValue() {
    const paletteTargetType = document.getElementById('palette-target-type');
    const paletteTargetItem = document.getElementById('palette-target-item');
    const paletteProperty = document.getElementById('palette-property');
    const paletteClearGroup = document.getElementById('palette-clear-group');
    const palettePropertyGroup = document.getElementById('palette-property-group');
    const paletteColorGroup = document.getElementById('palette-color-group');
    const paletteRoundingGroup = document.getElementById('palette-rounding-group');
    const paletteColor = document.getElementById('palette-color');
    const paletteRounding = document.getElementById('palette-rounding');
    const labelPaletteColor = document.getElementById('label-palette-color');

    if (!paletteTargetType || !paletteTargetItem) return;

    const isSpecific = paletteTargetType.value === 'specific';
    const target = paletteTargetItem.value;
    const prop = paletteProperty ? paletteProperty.value : 'bg';

    if (paletteClearGroup) paletteClearGroup.style.display = isSpecific ? 'flex' : 'none';

    function rgbToHexStr(arr) { if (!arr || arr.length < 3) return '#1e1e23'; return '#' + arr.map(x => x.toString(16).padStart(2, '0')).join(''); }

    if (!isSpecific) {
        if (palettePropertyGroup) palettePropertyGroup.style.display = 'none';
        if (paletteColorGroup) paletteColorGroup.style.display = 'block';
        if (paletteRoundingGroup) paletteRoundingGroup.style.display = 'none';

        const color = state.currentStyle[target];
        if (color && paletteColor) paletteColor.value = rgbToHexStr(color);
    } else {
        if (palettePropertyGroup) palettePropertyGroup.style.display = 'block';
        const override = state.currentStyle.instance_overrides ? state.currentStyle.instance_overrides[target] : null;

        if (prop === 'rounding') {
            if (paletteColorGroup) paletteColorGroup.style.display = 'none';
            if (paletteRoundingGroup) paletteRoundingGroup.style.display = 'block';
            if (paletteRounding) paletteRounding.value = (override && override.rounding !== undefined) ? override.rounding : 4;
        } else {
            if (paletteColorGroup) paletteColorGroup.style.display = 'block';
            if (paletteRoundingGroup) paletteRoundingGroup.style.display = 'none';
            if (labelPaletteColor) labelPaletteColor.textContent = prop === 'bg' ? state.currentLabels.label_bg_color : state.currentLabels.label_text_color;

            let color = override ? (prop === 'bg' ? override.bg : override.text) : null;
            if (paletteColor) paletteColor.value = color ? rgbToHexStr(color) : '#ffffff';
        }
    }
}
