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
            // Navigation
            up: [38, 29460],      // ArrowUp, VK_UP
            down: [40, 29461],    // ArrowDown, VK_DOWN
            left: [37, 29462],    // ArrowLeft, VK_LEFT
            right: [39, 29463],   // ArrowRight, VK_RIGHT
            action: [13, 29443],  // Enter, VK_ENTER

            // Back button (multiple variants for different Tizen versions)
            back: [10009, 4, 27], // VK_BACK (Tizen), Android back (4), Escape (27)

            // Media keys (some Samsung remotes map media buttons to navigation)
            // Note: 19 conflicts with Up on some models, so we check context
            media_play_pause: [415, 19],  // MediaPlayPause button
            media_fast_forward: [417],     // MediaFastForward button
            media_rewind: [412],           // MediaRewind button
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
        const ua = navigator.userAgent.toLowerCase();

        // Log user agent for debugging
        console.log('[TV-PAL] User Agent:', ua);

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

        // Check for Chromecast with Google TV
        // Chromecast user agent contains "chromecast" and is Android-based
        if (ua.includes('chromecast') ||
            (ua.includes('android') && ua.includes('aarch64'))) {
            console.log('[TV-PAL] Detected: Chromecast with Google TV');
            return Platform.ANDROID_TV; // Chromecast uses Android TV keycodes
        }

        // Check for Android TV / Fire TV
        // AFT = Amazon Fire TV, Nexus Player = Android TV
        if (ua.includes('aft') ||
            ua.includes('fire') ||
            ua.includes('silk') ||
            (ua.includes('android') && (ua.includes('tv') || ua.includes('aftn')))) {
            console.log('[TV-PAL] Detected: Amazon Fire TV / Android TV');
            return Platform.FIRETV;
        }

        // Generic Android TV detection
        if (ua.includes('android') && (ua.includes('tv') || ua.includes('nexus player'))) {
            console.log('[TV-PAL] Detected: Android TV');
            return Platform.ANDROID_TV;
        }

        // Check for Vizio
        if (ua.includes('vizio')) {
            console.log('[TV-PAL] Detected: Vizio');
            return Platform.VIZIO;
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

        // Special handling for keycode 19 conflict on Samsung Tizen
        // On some models, 19 = Up, on others 19 = MediaPlayPause
        // We prioritize Up since navigation is more important for gameplay
        let action = mapKeycodeToAction(keyCode);

        // Log ALL key events in debug mode, even unmapped ones
        if (debugMode) {
            console.log(`[TV-PAL] Keydown: keyCode=${keyCode}, code="${e.code}", key="${e.key}", action=${action || 'UNMAPPED'}`);
        }

        if (action) {
            // Forward to Rust IMMEDIATELY for maximum responsiveness
            forwardToRust(action, true);

            // Prevent default behavior IMMEDIATELY
            // This is critical for back button - must happen before any browser handling
            e.preventDefault();
            e.stopPropagation();
            e.stopImmediatePropagation();
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

            // Prevent default behavior for consistency
            e.preventDefault();
            e.stopPropagation();
            e.stopImmediatePropagation();
            return false;
        }

        return true;
    }

    /**
     * Register Tizen keys (Samsung-specific)
     * Different Tizen versions may have different key codes
     */
    function registerTizenKeys() {
        if (currentPlatform === Platform.TIZEN && window.tizen && window.tizen.tvinputdevice) {
            try {
                // Register media keys to ensure we receive them
                window.tizen.tvinputdevice.registerKeyBatch([
                    'MediaPlayPause',
                    'MediaFastForward',
                    'MediaRewind',
                    'ColorF0Red',
                    'ColorF1Green',
                    'ColorF2Yellow',
                    'ColorF3Blue'
                ]);
                console.log('[TV-PAL] Tizen media keys registered successfully');
            } catch (e) {
                console.warn('[TV-PAL] Tizen key registration failed:', e);
            }
        }
    }

    /**
     * Debug function to log all key events (helpful for identifying unknown keycodes)
     */
    function logAllKeyEvents(e) {
        if (!debugMode) return;

        const keyCode = e.keyCode || e.which;
        const action = mapKeycodeToAction(keyCode);

        console.log(`[TV-PAL KEY] keyCode=${keyCode}, code="${e.code}", key="${e.key}", type="${e.type}", action=${action || 'UNMAPPED'}`);
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

        // Special handling for back button to ensure maximum responsiveness
        // Listen on multiple phases to catch events that might be consumed
        window.addEventListener('keydown', function(e) {
            const keyCode = e.keyCode || e.which;
            // Check if this is a back button keycode (Tizen: 10009, Android: 4, Escape: 27)
            if (keyCode === 10009 || keyCode === 4 || keyCode === 27) {
                if (debugMode) {
                    console.log(`[TV-PAL BACK] Caught back button: keyCode=${keyCode}`);
                }
                // Immediately prevent default and forward to Rust
                e.preventDefault();
                e.stopPropagation();
                e.stopImmediatePropagation();
                forwardToRust('back', true);
                return false;
            }
        }, true);  // Use capture phase with highest priority

        // Also add backup listeners for debugging (catches events even if consumed by others)
        if (debugMode) {
            window.addEventListener('keydown', logAllKeyEvents, true);
            window.addEventListener('keyup', logAllKeyEvents, true);
            console.log('[TV-PAL] Debug mode enabled - all key events will be logged');
        }

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
