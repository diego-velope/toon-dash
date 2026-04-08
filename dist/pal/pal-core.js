/**
 * TV Platform Abstraction Layer (PAL) core for Toon Dash
 *
 * Load after: pal/namespace.js, pal/detect-platform.js, pal/platforms/*.js
 *
 * INTEGRATION:
 * 1. Load this script BEFORE the WASM module in index.html
 * 2. TV_PAL auto-initializes on DOMContentLoaded
 * 3. Key events map per platform and forward via window.mq_handle_*
 */
(function () {
    'use strict';

    var TDP = window.ToonDashPAL;
    var P = TDP.PLATFORM_IDS;

    var currentPlatform = null;
    var keyMapping = null;
    var isInitialized = false;
    var debugMode = true;

    function debugLog() {
        if (debugMode) {
            console.log.apply(console, arguments);
        }
    }

    function detectPlatform() {
        return TDP.detectPlatform();
    }

    function mapKeycodeToAction(keyCode) {
        if (!keyMapping) return null;
        for (var action in keyMapping) {
            if (!Object.prototype.hasOwnProperty.call(keyMapping, action)) continue;
            var codes = keyMapping[action];
            if (codes.indexOf(keyCode) !== -1) {
                return action;
            }
        }
        return null;
    }

    function forwardToRust(action, pressed) {
        debugLog('[PAL] Key: ' + action + ' = ' + pressed);

        var functionMap = {
            up: 'mq_handle_up',
            down: 'mq_handle_down',
            left: 'mq_handle_left',
            right: 'mq_handle_right',
            action: 'mq_handle_action',
            back: 'mq_handle_back'
        };

        var funcName = functionMap[action];
        if (!funcName) {
            console.warn('[TV-PAL] Unknown action:', action);
            return;
        }

        if (typeof window[funcName] === 'function') {
            try {
                window[funcName](pressed ? 1 : 0);
            } catch (e) {
                console.warn('[TV-PAL] Failed to call', funcName + ':', e);
            }
        } else if (debugMode) {
            console.warn('[TV-PAL] Function not available:', funcName);
        }
    }

    function isBackKeyCode(keyCode) {
        var backCodes = keyMapping && keyMapping.back ? keyMapping.back : [10009, 4, 27];
        return backCodes.indexOf(keyCode) !== -1;
    }

    function handleBackEvent(e, pressed) {
        var keyCode = e.keyCode || e.which;
        if (!isBackKeyCode(keyCode)) return false;

        debugLog(
            '[TV-PAL BACK] keyCode=' +
                keyCode +
                ', code="' +
                e.code +
                '", key="' +
                e.key +
                '", pressed=' +
                pressed
        );

        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        forwardToRust('back', pressed);
        return true;
    }

    function handleKeyDown(e) {
        var keyCode = e.keyCode || e.which;
        var action = mapKeycodeToAction(keyCode);

        debugLog(
            '[TV-PAL] Keydown: keyCode=' +
                keyCode +
                ', code="' +
                e.code +
                '", key="' +
                e.key +
                '", action=' +
                (action || 'UNMAPPED')
        );

        if (action) {
            if (action === 'back') {
                handleBackEvent(e, true);
            } else {
                forwardToRust(action, true);
                e.preventDefault();
                e.stopPropagation();
                e.stopImmediatePropagation();
            }
            return false;
        }

        return true;
    }

    function handleKeyUp(e) {
        var keyCode = e.keyCode || e.which;
        var action = mapKeycodeToAction(keyCode);

        debugLog(
            '[TV-PAL] Keyup: keyCode=' +
                keyCode +
                ', code="' +
                e.code +
                '", key="' +
                e.key +
                '", action=' +
                (action || 'UNMAPPED')
        );

        if (action) {
            if (action === 'back') {
                handleBackEvent(e, false);
            } else {
                forwardToRust(action, false);
                e.preventDefault();
                e.stopPropagation();
                e.stopImmediatePropagation();
            }
            return false;
        }

        return true;
    }

    function logAllKeyEvents(e) {
        if (!debugMode) return;

        var keyCode = e.keyCode || e.which;
        var action = mapKeycodeToAction(keyCode);

        debugLog(
            '[TV-PAL KEY] keyCode=' +
                keyCode +
                ', code="' +
                e.code +
                '", key="' +
                e.key +
                '", type="' +
                e.type +
                '", action=' +
                (action || 'UNMAPPED')
        );
    }

    function init(options) {
        options = options || {};

        if (isInitialized) {
            console.warn('[TV-PAL] Already initialized');
            return;
        }

        debugMode = options.debug || false;

        currentPlatform = detectPlatform();
        var impl =
            TDP.platforms[currentPlatform] || TDP.platforms[P.BROWSER];
        if (!impl) {
            console.error('[TV-PAL] No platform adapter for', currentPlatform);
            return;
        }

        keyMapping = impl.keyMapping;

        debugLog('[TV-PAL] Initializing for platform:', currentPlatform);
        debugLog('[TV-PAL] Key mappings:', keyMapping);

        if (typeof impl.registerKeys === 'function') {
            impl.registerKeys({ currentPlatform: currentPlatform });
        }

        window.addEventListener('keydown', handleKeyDown, true);
        window.addEventListener('keyup', handleKeyUp, true);

        window.addEventListener(
            'keydown',
            function (e) {
                if (handleBackEvent(e, true)) {
                    return false;
                }
            },
            true
        );

        if (debugMode) {
            window.addEventListener('keydown', logAllKeyEvents, true);
            window.addEventListener('keyup', logAllKeyEvents, true);
            debugLog('[TV-PAL] Debug mode enabled - all key events will be logged');
        }

        if (debugMode) {
            window.addEventListener(
                'keydown',
                function (e) {
                    var keyCode = e.keyCode || e.which;
                    console.log(
                        '[TV-PAL DEBUG] Raw keydown: keyCode=' +
                            keyCode +
                            ', code="' +
                            e.code +
                            '", key="' +
                            e.key +
                            '", defaultPrevented=' +
                            e.defaultPrevented
                    );
                },
                true
            );

            window.addEventListener('popstate', function () {
                console.log(
                    '[TV-PAL DEBUG] popstate event detected (back button pressed as navigation)'
                );
            });

            window.addEventListener('gamepadconnected', function (e) {
                console.log('[TV-PAL DEBUG] Gamepad connected:', e.gamepad);
            });
        }

        window.addEventListener('contextmenu', function (e) {
            e.preventDefault();
            e.stopPropagation();
            return false;
        });

        isInitialized = true;
        debugLog('[TV-PAL] Initialization complete');
    }

    /**
     * Teardown listeners and run platform host exit when available (Tizen, webOS, Android TV host).
     * @returns {boolean} true if caller may try window.close(); false if host handled app exit.
     */
    function shutdown() {
        if (isInitialized) {
            window.removeEventListener('keydown', handleKeyDown, true);
            window.removeEventListener('keyup', handleKeyUp, true);
            isInitialized = false;
        }

        var impl =
            currentPlatform &&
            (TDP.platforms[currentPlatform] || TDP.platforms[P.BROWSER]);

        if (impl && typeof impl.shutdownHost === 'function') {
            try {
                var hostHandledExit = impl.shutdownHost() === true;
                if (hostHandledExit) {
                    debugLog('[TV-PAL] Shutdown complete (host handled app exit)');
                    return false;
                }
            } catch (e) {
                console.warn('[TV-PAL] shutdownHost failed:', e);
                debugLog('[TV-PAL] Shutdown complete (host hook failed, fallback may run)');
                return true;
            }
        }

        debugLog('[TV-PAL] Shutdown complete');
        return true;
    }

    function getPlatform() {
        return currentPlatform;
    }

    function setDebug(enabled) {
        debugMode = enabled;
    }

    function _handleAndroidKeyEvent(keyCode, state) {
        debugLog(
            '[TV-PAL] _handleAndroidKeyEvent CALLED: keyCode=' + keyCode + ', state=' + state
        );

        var webKeyCode = keyCode;
        if (keyCode === 66) webKeyCode = 13;
        if (keyCode === 82) webKeyCode = 999;

        var action = mapKeycodeToAction(webKeyCode);
        if (action) {
            var pressed = state === 'down';
            forwardToRust(action, pressed);
            debugLog('[TV-PAL] Android key forwarded: ' + action + ' = ' + pressed);
        } else {
            debugLog(
                '[TV-PAL] Android key not mapped: keyCode=' +
                    keyCode +
                    ' (webKeyCode=' +
                    webKeyCode +
                    ')'
            );
        }
    }

    var TV_PAL = {
        init: init,
        shutdown: shutdown,
        getPlatform: getPlatform,
        setDebug: setDebug,
        _handleAndroidKeyEvent: _handleAndroidKeyEvent,
        _detectPlatform: detectPlatform,
        _mapKeycodeToAction: function (code) {
            return mapKeycodeToAction(code);
        }
    };

    window._handleAndroidKeyEvent = function (keyCode, state) {
        debugLog(
            '[TV-PAL GLOBAL] window._handleAndroidKeyEvent called with:',
            keyCode,
            state
        );
        if (TV_PAL && TV_PAL._handleAndroidKeyEvent) {
            TV_PAL._handleAndroidKeyEvent(keyCode, state);
        } else {
            console.error(
                '[TV-PAL] TV_PAL not initialized when Android called _handleAndroidKeyEvent'
            );
        }
    };

    debugLog(
        '[TV-PAL] Global functions registered. window._handleAndroidKeyEvent =',
        typeof window._handleAndroidKeyEvent
    );

    var _tvPalDebug = new URLSearchParams(window.location.search).has('debug');

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function () {
            TV_PAL.init({ debug: _tvPalDebug });
        });
    } else {
        TV_PAL.init({ debug: _tvPalDebug });
    }

    window.TV_PAL = TV_PAL;
})();
