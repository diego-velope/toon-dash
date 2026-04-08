/**
 * Shared namespace for modular TV PAL (Toon Dash).
 * Load first, before detect-platform.js and platform modules.
 */
(function (global) {
    'use strict';

    var PLATFORM_IDS = {
        TIZEN: 'tizen',
        WEBOS: 'webos',
        VIZIO: 'vizio',
        FIRETV: 'firetv',
        ANDROID_TV: 'android_tv',
        BROWSER: 'browser'
    };

    global.ToonDashPAL = global.ToonDashPAL || {};
    global.ToonDashPAL.PLATFORM_IDS = PLATFORM_IDS;
    global.ToonDashPAL.platforms = global.ToonDashPAL.platforms || {};

    global.ToonDashPAL.registerPlatform = function (id, impl) {
        if (!impl || !impl.keyMapping) {
            console.warn('[ToonDashPAL] registerPlatform: missing keyMapping for', id);
            return;
        }
        global.ToonDashPAL.platforms[id] = impl;
    };
})(typeof window !== 'undefined' ? window : this);
