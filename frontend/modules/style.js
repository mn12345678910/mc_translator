const { invoke } = window.__TAURI__.core;
import { state } from './state.js';

// 輔助函數：RGB 轉 Hex
function rgbToHexStr(arr) {
    if (!arr || arr.length < 3) return '#1e1e23';
    return '#' + arr.map((x) => x.toString(16).padStart(2, '0')).join('');
}

// 輔助函數：Hex 轉 RGB Array
export function hexToRgbArr(hex) {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return [r, g, b];
}

export function applyColors(style) {
    if (!style) return;
    const root = document.documentElement;
    const isDark = style.theme !== 'light';

    // --- [基礎與強調顏色] ---
    const bg = isDark ? style.dark_bg : style.light_bg;
    const text = isDark ? style.dark_text : style.light_text;
    const accent = isDark ? style.dark_accent : style.light_accent;
    const danger = isDark ? style.dark_danger : style.light_danger;

    if (bg) root.style.setProperty('--bg-color', `rgb(${bg[0]},${bg[1]},${bg[2]})`);
    if (text) root.style.setProperty('--text-color', `rgb(${text[0]},${text[1]},${text[2]})`);
    if (accent) root.style.setProperty('--accent-color', `rgb(${accent[0]},${accent[1]},${accent[2]})`);
    if (danger) root.style.setProperty('--danger-color', `rgb(${danger[0]},${danger[1]},${danger[2]})`);

    // --- [間距系統] ---
    if (style.space_sm !== undefined) root.style.setProperty('--space-sm', `${style.space_sm}px`);
    if (style.space_md !== undefined) root.style.setProperty('--space-md', `${style.space_md}px`);
    if (style.space_lg !== undefined) root.style.setProperty('--space-lg', `${style.space_lg}px`);

    // --- [細部元件] ---
    const label = isDark ? style.dark_label : style.light_label;
    const textMuted = isDark ? style.dark_text_muted : style.light_text_muted;
    const btnBg = isDark ? style.dark_btn_bg : style.light_btn_bg;
    const btnText = isDark ? style.dark_btn_text : style.light_btn_text;
    const inputBg = isDark ? style.dark_input_bg : style.light_input_bg;
    const listBg = isDark ? style.dark_list_bg : style.light_list_bg;
    const tabActive = isDark ? style.dark_tab_active : style.light_tab_active;
    const tabInactive = isDark ? style.dark_tab_inactive : style.light_tab_inactive;
    const headerBg = isDark ? style.dark_header_bg : style.light_header_bg;
    const borderColor = isDark ? style.dark_border_color : style.light_border_color;
    const hoverBg = isDark ? style.dark_hover_bg : style.light_hover_bg;
    const sliderBg = isDark ? style.dark_slider_bg : style.light_slider_bg;
    const sliderThumb = isDark ? style.dark_slider_thumb : style.light_slider_thumb;
    const switchBg = isDark ? style.dark_switch_bg : style.light_switch_bg;
    const progressBg = isDark ? style.dark_progress_bg : style.light_progress_bg;

    if (label) root.style.setProperty('--label-color', `rgb(${label[0]},${label[1]},${label[2]})`);
    if (textMuted) root.style.setProperty('--text-muted', `rgb(${textMuted[0]},${textMuted[1]},${textMuted[2]})`);
    if (btnBg) root.style.setProperty('--btn-bg', `rgb(${btnBg[0]},${btnBg[1]},${btnBg[2]})`);
    if (btnText) root.style.setProperty('--btn-text', `rgb(${btnText[0]},${btnText[1]},${btnText[2]})`);
    if (inputBg) root.style.setProperty('--input-bg', `rgb(${inputBg[0]},${inputBg[1]},${inputBg[2]})`);
    if (listBg) root.style.setProperty('--list-bg', `rgb(${listBg[0]},${listBg[1]},${listBg[2]})`);
    if (tabActive) root.style.setProperty('--tab-active-bg', `rgb(${tabActive[0]},${tabActive[1]},${tabActive[2]})`);
    if (tabInactive) root.style.setProperty('--tab-inactive-bg', `rgb(${tabInactive[0]},${tabInactive[1]},${tabInactive[2]})`);
    if (headerBg) root.style.setProperty('--header-bg', `rgb(${headerBg[0]},${headerBg[1]},${headerBg[2]})`);
    if (borderColor) {
        root.style.setProperty('--border-color', `rgb(${borderColor[0]},${borderColor[1]},${borderColor[2]})`);
        const alpha = style.border_alpha !== undefined ? style.border_alpha : 0.15;
        root.style.setProperty('--border-light', `rgba(${borderColor[0]},${borderColor[1]},${borderColor[2]},${alpha})`);
    }
    if (hoverBg) root.style.setProperty('--hover-bg', `rgb(${hoverBg[0]},${hoverBg[1]},${hoverBg[2]})`);
    if (sliderBg) root.style.setProperty('--slider-bg', `rgb(${sliderBg[0]},${sliderBg[1]},${sliderBg[2]})`);
    if (sliderThumb) root.style.setProperty('--slider-thumb', `rgb(${sliderThumb[0]},${sliderThumb[1]},${sliderThumb[2]})`);
    if (switchBg) root.style.setProperty('--switch-bg', `rgb(${switchBg[0]},${switchBg[1]},${switchBg[2]})`);
    if (progressBg) root.style.setProperty('--progress-bg', `rgb(${progressBg[0]},${progressBg[1]},${progressBg[2]})`);

    // --- [面板與背景透明度] ---
    if (bg) {
        const pAlpha = style.panel_alpha !== undefined ? style.panel_alpha : 0.03;
        const bAlpha = style.backdrop_alpha !== undefined ? style.backdrop_alpha : 0.6;
        root.style.setProperty('--panel-bg', `rgba(${bg[0]},${bg[1]},${bg[2]},${pAlpha})`);
        root.style.setProperty('--backdrop-bg', `rgba(${bg[0]},${bg[1]},${bg[2]},${bAlpha})`);
    }

    // --- [佈局基礎設定] ---
    if (style.font_size) root.style.setProperty('--font-size', `${style.font_size}px`);
    if (style.btn_rounding_enabled !== false) {
        root.style.setProperty('--border-radius', `${style.btn_rounding_value}px`);
    } else {
        root.style.setProperty('--border-radius', '0px');
    }

    // --- [動畫與進度條] ---
    const progressBar = document.getElementById('progress-bar');
    const batchProgressBar = document.getElementById('batch-progress-bar');
    if (style.progress_pulse_enabled && progressBar) {
        progressBar.style.animation = `pulse ${2.0 / Math.max(0.1, style.progress_pulse_speed)}s infinite`;
    } else if (progressBar) {
        progressBar.style.animation = 'none';
    }

    const bars = [progressBar, batchProgressBar];
    bars.forEach(bar => {
        if (!bar) return;
        bar.classList.remove('aurora', 'neon');
        if (style.progress_style === 'aurora') bar.classList.add('aurora');
        if (style.progress_style === 'neon') bar.classList.add('neon');
    });

    // --- [實例覆寫] ---
    if (style.instance_overrides) {
        for (const [id, overrides] of Object.entries(style.instance_overrides)) {
            const el = document.getElementById(id);
            if (!el) continue;
            if (overrides.bg) el.style.backgroundColor = `rgb(${overrides.bg[0]},${overrides.bg[1]},${overrides.bg[2]})`;
            if (overrides.text) el.style.color = `rgb(${overrides.text[0]},${overrides.text[1]},${overrides.text[2]})`;
            if (overrides.rounding !== undefined) el.style.borderRadius = `${overrides.rounding}px`;
        }
    }
}

export function updatePaletteValue() {
    const paletteTargetType = document.getElementById('palette-target-type');
    const paletteTargetItem = document.getElementById('palette-target-item');
    const paletteProperty = document.getElementById('palette-property');
    const palettePropertyGroup = document.getElementById('palette-property-group');
    const paletteColorGroup = document.getElementById('palette-color-group');
    const paletteNumberGroup = document.getElementById('palette-number-group');
    const paletteClearGroup = document.getElementById('palette-clear-group');
    const paletteNumber = document.getElementById('palette-number');
    const paletteColor = document.getElementById('palette-color');
    const labelPaletteNumber = document.getElementById('label-palette-number');
    const labelPaletteColor = document.getElementById('label-palette-color');

    if (!paletteTargetType || !paletteTargetItem || !paletteProperty) return;

    const isSpecific = paletteTargetType.value === 'specific';
    const target = paletteTargetItem.value;
    const prop = paletteProperty.value;

    // 更新群組顯示狀態
    if (paletteClearGroup) paletteClearGroup.style.display = isSpecific ? 'flex' : 'none';
    if (palettePropertyGroup) palettePropertyGroup.style.display = isSpecific ? 'block' : 'none';

    // 判斷是否為數值型項目
    const isNumberItem = target.startsWith('space_') || target.endsWith('_alpha') || target === 'font_size' || (isSpecific && prop === 'rounding');

    if (paletteColorGroup) paletteColorGroup.style.display = isNumberItem ? 'none' : 'block';
    if (paletteNumberGroup) paletteNumberGroup.style.display = isNumberItem ? 'block' : 'none';

    if (isNumberItem) {
        // 設定 Label 與數值
        if (labelPaletteNumber) {
            if (target.startsWith('space_')) {
                labelPaletteNumber.textContent = (state.currentLabels && state.currentLabels.palette_label_spacing) ? state.currentLabels.palette_label_spacing : 'Spacing (px)';
            } else if (target.endsWith('_alpha')) {
                labelPaletteNumber.textContent = (state.currentLabels && state.currentLabels.palette_label_alpha) ? state.currentLabels.palette_label_alpha : 'Alpha (0.0-1.0)';
            } else if (target === 'font_size') {
                labelPaletteNumber.textContent = (state.currentLabels && state.currentLabels.label_font_size) ? state.currentLabels.label_font_size : 'Font Size (px)';
            } else {
                labelPaletteNumber.textContent = (state.currentLabels && state.currentLabels.palette_label_rounding) ? state.currentLabels.palette_label_rounding : 'Rounding (px)';
            }
        }

        let val = 0;
        if (isSpecific) {
            const ov = state.currentStyle.instance_overrides ? state.currentStyle.instance_overrides[target] : null;
            val = ov ? (ov.rounding || 4) : 4;
        } else {
            val = state.currentStyle[target] || 0;
        }
        if (paletteNumber) {
            paletteNumber.value = val;
            paletteNumber.step = target.endsWith('_alpha') ? '0.01' : '1';
        }
    } else {
        // 設定顏色預覽
        if (labelPaletteColor) {
            labelPaletteColor.textContent = prop === 'bg'
                ? ((state.currentLabels && state.currentLabels.label_bg_color) ? state.currentLabels.label_bg_color : 'Background')
                : ((state.currentLabels && state.currentLabels.label_text_color) ? state.currentLabels.label_text_color : 'Text');
        }

        let color = null;
        if (isSpecific) {
            const ov = state.currentStyle.instance_overrides ? state.currentStyle.instance_overrides[target] : null;
            color = ov ? (prop === 'bg' ? ov.bg : ov.text) : null;
        } else {
            color = state.currentStyle[target];
        }
        if (paletteColor) paletteColor.value = color ? rgbToHexStr(color) : '#ffffff';
    }

    // 處理特定元件的文字選項隱藏
    const noTextItems = ['progress-bar', 'batch-progress-bar'];
    if (isSpecific) {
        Array.from(paletteProperty.options).forEach((opt) => {
            if (opt.value === 'text') {
                const hidden = noTextItems.includes(target);
                opt.style.display = hidden ? 'none' : 'block';
                opt.disabled = hidden;
                if (hidden && paletteProperty.value === 'text') paletteProperty.value = 'bg';
            }
        });
    }
}

export async function loadStyle() {
    try {
        const style = await invoke('get_style_config');
        state.currentStyle = style;
        applyColors(style);

        // --- 同步常規控制項 ---
        const controls = {
            'font-size': style.font_size,
            'btn-rounding-value': style.btn_rounding_value,
            'pulse-speed': style.progress_pulse_speed,
            'progress-style': style.progress_style || 'default',
        };
        for (const [id, val] of Object.entries(controls)) {
            const el = document.getElementById(id);
            if (el) el.value = val;
        }

        const checks = {
            'chk-btn-rounding': style.btn_rounding_enabled,
            'chk-pulse': style.progress_pulse_enabled,
        };
        for (const [id, val] of Object.entries(checks)) {
            const el = document.getElementById(id);
            if (el) el.checked = !!val;
        }

        // --- 同步顏色選擇器 (支援 Legacy IDs) ---
        const colorMaps = {
            'color-bg': style.dark_bg,
            'color-text': style.dark_text,
            'color-accent': style.dark_accent,
            'color-danger': style.dark_danger,
        };
        for (const [id, val] of Object.entries(colorMaps)) {
            const el = document.getElementById(id);
            if (el && val) el.value = rgbToHexStr(val);
        }

        if (typeof updatePaletteValue === 'function') {
            updatePaletteValue();
        }
    } catch (e) {
        console.error('載入樣式配置失敗:', e);
    }
}

export async function saveStyle() {
    try {
        const fontSize = document.getElementById('font-size');
        const chkBtnRounding = document.getElementById('chk-btn-rounding');
        const btnRoundingValue = document.getElementById('btn-rounding-value');
        const chkPulse = document.getElementById('chk-pulse');
        const pulseSpeed = document.getElementById('pulse-speed');
        const progressStyle = document.getElementById('progress-style');

        if (fontSize) state.currentStyle.font_size = parseInt(fontSize.value) || 16;
        if (chkBtnRounding) state.currentStyle.btn_rounding_enabled = chkBtnRounding.checked;
        if (btnRoundingValue) state.currentStyle.btn_rounding_value = parseFloat(btnRoundingValue.value) || 4.0;
        if (chkPulse) state.currentStyle.progress_pulse_enabled = chkPulse.checked;
        if (pulseSpeed) state.currentStyle.progress_pulse_speed = parseFloat(pulseSpeed.value) || 1.0;
        if (progressStyle) state.currentStyle.progress_style = progressStyle.value || 'default';

        // --- 讀取 Legacy 顏色 (僅當存在時) ---
        const colorMaps = {
            'color-bg': 'dark_bg',
            'color-text': 'dark_text',
            'color-accent': 'dark_accent',
            'color-danger': 'dark_danger',
        };
        for (const [id, key] of Object.entries(colorMaps)) {
            const el = document.getElementById(id);
            if (el && el.value) {
                state.currentStyle[key] = hexToRgbArr(el.value);
            }
        }

        applyColors(state.currentStyle);
        await invoke('save_style_config', { config: state.currentStyle });
    } catch (e) {
        console.error('儲存樣式配置失敗:', e);
    }
}

export async function restoreDefaultStyle() {
    try {
        const defaultStyle = await invoke('get_default_style_config');

        state.currentStyle = {
            ...defaultStyle,
            show_palette_settings: state.currentStyle.show_palette_settings
        };

        applyColors(state.currentStyle);
        await invoke('save_style_config', { config: state.currentStyle });

        const fontSize = document.getElementById('font-size');
        const chkBtnRounding = document.getElementById('chk-btn-rounding');
        const btnRoundingValue = document.getElementById('btn-rounding-value');
        const chkPulse = document.getElementById('chk-pulse');
        const pulseSpeed = document.getElementById('pulse-speed');
        const progressStyle = document.getElementById('progress-style');

        if (fontSize) fontSize.value = state.currentStyle.font_size;
        if (chkBtnRounding) chkBtnRounding.checked = state.currentStyle.btn_rounding_enabled;
        if (btnRoundingValue) btnRoundingValue.value = state.currentStyle.btn_rounding_value;
        if (chkPulse) chkPulse.checked = state.currentStyle.progress_pulse_enabled;
        if (pulseSpeed) pulseSpeed.value = state.currentStyle.progress_pulse_speed;
        if (progressStyle) progressStyle.value = state.currentStyle.progress_style || 'default';

        if (typeof updatePaletteValue === 'function') {
            updatePaletteValue();
        }

        const msg = (state.currentLabels && state.currentLabels.status_style_restored) ? state.currentLabels.status_style_restored : '介面樣式已恢復預設';
        console.log(msg);
    } catch (e) {
        console.error('恢復樣式預設失敗:', e);
    }
}
