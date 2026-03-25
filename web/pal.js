/**
 * Platform Abstraction Layer (PAL) for Toon Dash
 * 
 * Platform keycodes:
 * - Samsung Tizen: Back = 10009
 * - LG webOS: Back = 461
 * - Fire TV: Back = 8
 * - Browser: Back = Escape
 */

const PAL = (function() {
    'use strict';

    const Platform = {
        TIZEN: 'tizen',
        WEBOS: 'webos',
        FIRETV: 'firetv',
        BROWSER: 'browser'
    };

    const keyMappings = {
        [Platform.TIZEN]: {
            Up: 38, Down: 40, Left: 37, Right: 39,
            Enter: 13, Back: 10009
        },
        [Platform.WEBOS]: {
            Up: 38, Down: 40, Left: 37, Right: 39,
            Enter: 13, Back: 461
        },
        [Platform.FIRETV]: {
            Up: 38, Down: 40, Left: 37, Right: 39,
            Enter: 13, Back: 8
        },
        [Platform.BROWSER]: {
            Up: 38, Down: 40, Left: 37, Right: 39,
            Enter: 13, Back: 27
        }
    };

    let currentPlatform = null;
    let keyMapping = null;

    function detect() {
        const ua = navigator.userAgent.toLowerCase();

        if (ua.includes('tizen') || window.tizen) {
            return Platform.TIZEN;
        }
        if (ua.includes('webos') || ua.includes('web0s')) {
            return Platform.WEBOS;
        }
        if (ua.includes('aft') || ua.includes('fire tv')) {
            return Platform.FIRETV;
        }
        return Platform.BROWSER;
    }

    function init() {
        currentPlatform = detect();
        keyMapping = keyMappings[currentPlatform];

        if (currentPlatform === Platform.TIZEN && window.tizen) {
            try {
                window.tizen.tvinputdevice.registerKeyBatch([
                    'MediaPlay', 'MediaPause', 'MediaFastForward', 'MediaRewind'
                ]);
                console.log('[PAL] Tizen keys registered');
            } catch (e) {
                console.warn('[PAL] Tizen key registration failed:', e);
            }
        }

        console.log('[PAL] Platform:', currentPlatform);
    }

    return {
        init: init,

        getPlatform: function() {
            return currentPlatform;
        },

        getKeyMapping: function() {
            return keyMapping;
        },

        mapKey: function(keyCode) {
            for (const [name, code] of Object.entries(keyMapping)) {
                if (code === keyCode) return name;
            }
            return null;
        },

        isTV: function() {
            return currentPlatform !== Platform.BROWSER;
        }
    };
})();

document.addEventListener('DOMContentLoaded', PAL.init);