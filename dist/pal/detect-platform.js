/**
 * TV platform detection (user agent + host globals).
 * Depends on: pal/namespace.js
 */
(function (global) {
    'use strict';

    var P = global.ToonDashPAL.PLATFORM_IDS;

    function detectPlatform() {
        var rawUa = navigator.userAgent || '';
        var ua = rawUa.toLowerCase();

        console.log('[TV-PAL] User Agent:', ua);

        if (global.tizen) {
            console.log('[TV-PAL] Detected: Samsung Tizen');
            return P.TIZEN;
        }

        if (global.webOS) {
            console.log('[TV-PAL] Detected: LG webOS');
            return P.WEBOS;
        }

        // Lightning-style Fire TV signatures before generic Android TV heuristics
        if (
            rawUa.search(/AmazonPlatform/i) > -1 ||
            rawUa.search(/AFTMM/i) > -1 ||
            rawUa.search(/AFTKM/i) > -1
        ) {
            console.log('[TV-PAL] Detected: Amazon Fire TV (UA signature)');
            return P.FIRETV;
        }

        if (ua.includes('chromecast') || (ua.includes('android') && ua.includes('aarch64'))) {
            console.log('[TV-PAL] Detected: Chromecast with Google TV');
            return P.ANDROID_TV;
        }

        if (
            ua.includes('aft') ||
            ua.includes('fire') ||
            ua.includes('silk') ||
            (ua.includes('android') && (ua.includes('tv') || ua.includes('aftn')))
        ) {
            console.log('[TV-PAL] Detected: Amazon Fire TV / Android TV');
            return P.FIRETV;
        }

        if (ua.includes('android') && (ua.includes('tv') || ua.includes('nexus player'))) {
            console.log('[TV-PAL] Detected: Android TV');
            return P.ANDROID_TV;
        }

        if (ua.includes('vizio')) {
            console.log('[TV-PAL] Detected: Vizio');
            return P.VIZIO;
        }

        console.log('[TV-PAL] Detected: Browser (desktop testing mode)');
        return P.BROWSER;
    }

    global.ToonDashPAL.detectPlatform = detectPlatform;
})(typeof window !== 'undefined' ? window : this);
