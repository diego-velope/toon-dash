/**
 * TV Platform Abstraction Layer (PAL) for Toon Dash
 *
 * This JavaScript module provides platform-specific keycode normalization
 * for TV platforms and forwards input events to Rust via Macroquad.
 *
 * SUPPORTED PLATFORMS:
 * - Samsung Tizen (Back: 10009)
 * - LG webOS (Back: 461)
 * - Vizio (Back: 8)
 * - Fire TV / Android TV (Back: 8, Enter: 23)
 * - Generic Browser (Back: Escape, Enter: Enter)
 *
 * INTEGRATION:
 * 1. Load this script BEFORE the WASM module in index.html
 * 2. TV_PAL auto-initializes on DOMContentLoaded
 * 3. The PAL will automatically register keydown/keyup listeners
 * 4. Key events are mapped and sent to Rust via window.mq_handle_* functions
 */

const TV_PAL = (function() {
    'use strict';

    // Platform detection constants
    const Platform = {
        TIZEN: 'tizen',
        WEBOS: 'webos',
        VIZIO: 'vizio',
        FIRETV: 'firetv',
        ANDROID_TV: 'android_tv',
        BROWSER: 'browser'
    };

    // Platform-specific keycode mappings
    // Each platform has different keycodes for the same logical buttons
    const keyMappings = {
        [Platform.TIZEN]: {
            // Samsung Tizen keycodes
            up: [38, 29460],      // ArrowUp, VK_UP
            down: [40, 29461],    // ArrowDown, VK_DOWN
            left: [37, 29462],    // ArrowLeft, VK_LEFT
            right: [39, 29463],   // ArrowRight, VK_RIGHT
            action: [13, 29443],  // Enter, VK_ENTER
            back: [10009],        // VK_BACK
        },
        [Platform.WEBOS]: {
            // LG webOS keycodes
            up: [38],
            down: [40],
            left: [37],
            right: [39],
            action: [13],
            back: [461],          // webOS Back button
        },
        [Platform.VIZIO]: {
            // Vizio keycodes
            up: [38],
            down: [40],
            left: [37],
            right: [39],
            action: [13],
            back: [8],            // Backspace
        },
        [Platform.FIRETV]: {
            // Fire TV keycodes
            up: [19],
            down: [20],
            left: [21],
            right: [22],
            action: [23, 66],     // DPAD_CENTER, ENTER
            back: [4, 27, 8, 11],         // Back button (API 19+), Backspace
        },
        [Platform.ANDROID_TV]: {
            // Generic Android TV keycodes
            up: [19],
            down: [20],
            left: [21],
            right: [22],
            action: [23, 66],
            back: [4, 27, 8, 111],  // Standard back (4), Escape (27), Backspace (8), some devices use 111
        },
        [Platform.BROWSER]: {
            // Browser/desktop keycodes (for testing)
            up: [38, 87],         // ArrowUp, W
            down: [40, 83],       // ArrowDown, S
            left: [37, 65],       // ArrowLeft, A
            right: [39, 68],      // ArrowRight, D
            action: [13, 32],     // Enter, Space
            back: [4, 27, 8, 11],        // Escape, Backspace
        }
    };

    let currentPlatform = null;
    let keyMapping = null;
    let isInitialized = false;
    let debugMode = true;  // Enable debug mode to capture Chromecast back keycode

    /**
     * Detect the current TV platform
     * Uses multiple methods: window APIs, user-agent, feature detection
     */
    function detectPlatform() {
        // Check for Tizen first (Samsung)
        if (window.tizen) {
            console.log('[TV-PAL] Detected: Samsung Tizen');
            return Platform.TIZEN;
        }

        // Check for webOS (LG)
        if (window.webOS) {
            console.log('[TV-PAL] Detected: LG webOS');
            return Platform.WEBOS;
        }

        // Check user agent for specific platforms
        const ua = navigator.userAgent.toLowerCase();

        if (ua.includes('vizio')) {
            console.log('[TV-PAL] Detected: Vizio');
            return Platform.VIZIO;
        }

        if (ua.includes('aft') || ua.includes('fire tv') || ua.includes('silk')) {
            console.log('[TV-PAL] Detected: Amazon Fire TV');
            return Platform.FIRETV;
        }

        if (ua.includes('android') && (ua.includes('tv') || ua.includes('aft'))) {
            console.log('[TV-PAL] Detected: Android TV');
            return Platform.ANDROID_TV;
        }

        // Default to browser (for desktop testing)
        console.log('[TV-PAL] Detected: Browser (desktop testing mode)');
        return Platform.BROWSER;
    }

    /**
     * Map a raw keycode to a logical action
     */
    function mapKeycodeToAction(keyCode) {
        for (const [action, codes] of Object.entries(keyMapping)) {
            if (codes.includes(keyCode)) {
                return action;
            }
        }
        return null;
    }

    /**
     * Forward an input event to Rust via Macroquad's plugin system
     * Each action has its own function exposed to JavaScript
     */
    function forwardToRust(action, pressed) {
        if (debugMode) {
            console.log(`[TV-PAL] Key: ${action} = ${pressed}`);
        }

        // Map action to the corresponding Rust function
        // These functions are exposed via Macroquad's plugin system
        const functionMap = {
            'up': 'mq_handle_up',
            'down': 'mq_handle_down',
            'left': 'mq_handle_left',
            'right': 'mq_handle_right',
            'action': 'mq_handle_action',
            'back': 'mq_handle_back'
        };

        const funcName = functionMap[action];
        if (!funcName) {
            console.warn('[TV-PAL] Unknown action:', action);
            return;
        }

        // Call the Rust function through the window object (Macroquad exposes functions here)
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

    /**
     * Keydown event handler
     */
    function handleKeyDown(e) {
        const keyCode = e.keyCode || e.which;

        // Log ALL key events in debug mode, even unmapped ones
        if (debugMode) {
            const action = mapKeycodeToAction(keyCode);
            console.log(`[TV-PAL] Keydown: keyCode=${keyCode}, code="${e.code}", key="${e.key}", action=${action || 'UNMAPPED'}`);
        }

        const action = mapKeycodeToAction(keyCode);

        if (action) {
            // Forward to Rust
            forwardToRust(action, true);

            // Prevent default behavior for TV-specific keys
            // This prevents the browser from handling Back, etc.
            e.preventDefault();
            e.stopPropagation();
            return false;
        }

        return true;
    }

    /**
     * Keyup event handler
     */
    function handleKeyUp(e) {
        const keyCode = e.keyCode || e.which;
        const action = mapKeycodeToAction(keyCode);

        if (debugMode) {
            console.log(`[TV-PAL] Keyup: keyCode=${keyCode}, code="${e.code}", key="${e.key}", action=${action || 'UNMAPPED'}`);
        }

        if (action) {
            forwardToRust(action, false);
            e.preventDefault();
            e.stopPropagation();
            return false;
        }

        return true;
    }

    /**
     * Register Tizen keys (Samsung-specific)
     */
    function registerTizenKeys() {
        if (currentPlatform === Platform.TIZEN && window.tizen && window.tizen.tvinputdevice) {
            try {
                window.tizen.tvinputdevice.registerKeyBatch([
                    'MediaPlayPause',
                    'MediaFastForward',
                    'MediaRewind',
                    'ColorF0Red',
                    'ColorF1Green',
                    'ColorF2Yellow',
                    'ColorF3Blue'
                ]);
                console.log('[TV-PAL] Tizen media keys registered');
            } catch (e) {
                console.warn('[TV-PAL] Tizen key registration failed:', e);
            }
        }
    }

    /**
     * Initialize the TV PAL
     */
    function init(options = {}) {
        if (isInitialized) {
            console.warn('[TV-PAL] Already initialized');
            return;
        }

        debugMode = options.debug || false;

        // Detect platform and load key mappings
        currentPlatform = detectPlatform();
        keyMapping = keyMappings[currentPlatform];

        console.log('[TV-PAL] Initializing for platform:', currentPlatform);
        console.log('[TV-PAL] Key mappings:', keyMapping);

        // Register platform-specific keys
        registerTizenKeys();

        // Attach event listeners
        // Use capture phase to intercept keys before the browser handles them
        window.addEventListener('keydown', handleKeyDown, true);
        window.addEventListener('keyup', handleKeyUp, true);

        // Also add a backup listener to catch events that might be consumed by other handlers
        // This helps debug TV-specific key events
        if (debugMode) {
            window.addEventListener('keydown', function(e) {
                const keyCode = e.keyCode || e.which;
                console.log(`[TV-PAL DEBUG] Raw keydown: keyCode=${keyCode}, code="${e.code}", key="${e.key}", defaultPrevented=${e.defaultPrevented}`);
            }, true);

            // Listen for popstate (back button navigation)
            window.addEventListener('popstate', function(e) {
                console.log('[TV-PAL DEBUG] popstate event detected (back button pressed as navigation)');
            });

            // Try to catch gamepad events (some remotes register as gamepads)
            window.addEventListener('gamepadconnected', function(e) {
                console.log('[TV-PAL DEBUG] Gamepad connected:', e.gamepad);
            });
        }

        // Prevent context menu on long press (common on TV remotes)
        window.addEventListener('contextmenu', function(e) {
            e.preventDefault();
            e.stopPropagation();
            return false;
        });

        isInitialized = true;
        console.log('[TV-PAL] Initialization complete');
    }

    /**
     * Cleanup and remove event listeners
     */
    function shutdown() {
        if (!isInitialized) return;

        window.removeEventListener('keydown', handleKeyDown, true);
        window.removeEventListener('keyup', handleKeyUp, true);

        isInitialized = false;
        console.log('[TV-PAL] Shutdown complete');
    }

    /**
     * Get current platform (for debugging/display)
     */
    function getPlatform() {
        return currentPlatform;
    }

    /**
     * Enable/disable debug logging
     */
    function setDebug(enabled) {
        debugMode = enabled;
    }

    /**
     * Handle key events from Android wrapper
     * Called by AndroidJsInterface.onKeyEvent() when keys are pressed in the Android app
     *
     * @param {number} keyCode - The Android keycode (Android keycode constants)
     * @param {string} state - Either "down" or "up"
     */
    function _handleAndroidKeyEvent(keyCode, state) {
        console.log(`[TV-PAL] _handleAndroidKeyEvent CALLED: keyCode=${keyCode}, state=${state}`);

        // Map Android keycodes to web keycodes
        const androidToWebMap = {
            4: 4,     // KEYCODE_BACK → web back
            19: 19,   // KEYCODE_DPAD_UP
            20: 20,   // KEYCODE_DPAD_DOWN
            21: 21,   // KEYCODE_DPAD_LEFT
            22: 22,   // KEYCODE_DPAD_RIGHT
            23: 23,   // KEYCODE_DPAD_CENTER
            66: 13,   // KEYCODE_ENTER → Enter (13)
            82: 999,  // KEYCODE_MENU (Fire TV menu button)
            27: 27,   // KEYCODE_MEDIA_PLAY_PAUSE (use actual value)
        };

        // For Android keycodes, we need to handle them directly
        // Most D-pad keys already match web keycodes
        let webKeyCode = keyCode;

        // Special mapping for certain keys
        if (keyCode === 66) webKeyCode = 13;  // Enter
        if (keyCode === 82) webKeyCode = 999;  // Menu button - ignore or handle specially

        const action = mapKeycodeToAction(webKeyCode);
        if (action) {
            const pressed = state === 'down';
            forwardToRust(action, pressed);

            console.log(`[TV-PAL] Android key forwarded: ${action} = ${pressed}`);
        } else {
            console.log(`[TV-PAL] Android key not mapped: keyCode=${keyCode} (webKeyCode=${webKeyCode})`);
        }
    }

    // Public API
    return {
        init,
        shutdown,
        getPlatform,
        setDebug,
        _handleAndroidKeyEvent,  // Called by Android wrapper

        // Expose for testing/debugging
        _detectPlatform: detectPlatform,
        _mapKeycodeToAction: mapKeycodeToAction,
    };
})();

// Also expose a global function for Android to call directly (more reliable)
window._handleAndroidKeyEvent = function(keyCode, state) {
    console.log('[TV-PAL GLOBAL] window._handleAndroidKeyEvent called with:', keyCode, state);
    if (TV_PAL && TV_PAL._handleAndroidKeyEvent) {
        TV_PAL._handleAndroidKeyEvent(keyCode, state);
    } else {
        console.error('[TV-PAL] TV_PAL not initialized when Android called _handleAndroidKeyEvent');
    }
};

console.log('[TV-PAL] Global functions registered. window._handleAndroidKeyEvent =', typeof window._handleAndroidKeyEvent);

// Auto-initialize on DOMContentLoaded
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function() {
        TV_PAL.init({ debug: false });
    });
} else {
    TV_PAL.init({ debug: false });
}

// Export to global scope
window.TV_PAL = TV_PAL;
