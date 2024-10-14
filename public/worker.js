// Workers have their own scope and cannot directly access functions/objects in the global scope.
// They also cannot use ES6 modules.
importScripts("./bevy_in_web_worker.js");

// Destructure the imported functions from the wasm_bindgen object
const {
  init_bevy_app,
  is_preparation_completed,
  create_window_by_offscreen_canvas,
  enter_frame,
  mouse_move,
  set_hover,
  set_selection,
  left_bt_down,
  left_bt_up,
  set_auto_animation,
} = wasm_bindgen;

// Initialize variables for app state
let appHandle = 0;
let initFinished = 0;
let isStoppedRunning = false;
let renderBlockTime = 1;

async function init_wasm_in_worker() {
  // Load the WebAssembly file
  await wasm_bindgen("./bevy_in_web_worker_bg.wasm");

  // Create the Bevy app and store its handle
  appHandle = init_bevy_app();

  // Listen for messages from the main thread
  self.onmessage = async (event) => {
    let data = event.data;
    switch (data.ty) {
      case "init":
        // Initialize the app window with the provided canvas
        let canvas = data.canvas;
        createWorkerAppWindow(canvas, data.devicePixelRatio);
        break;

      case "startRunning":
        // Resume the animation loop if it was stopped
        if (isStoppedRunning) {
          isStoppedRunning = false;
          requestAnimationFrame(enterFrame);
        }
        break;

      case "stopRunning":
        // Stop the animation loop
        isStoppedRunning = true;
        break;

      case "mousemove":
        // Handle mouse movement
        mouse_move(appHandle, data.x, data.y);
        break;

      case "hover":
        // Set hover (highlight) effect
        set_hover(appHandle, data.list);
        break;

      case "select":
        // Set selection effect
        set_selection(appHandle, data.list);
        break;

      case "leftBtDown":
        // Handle left mouse button press
        left_bt_down(appHandle, data.pickItem, data.x, data.y);
        break;

      case "leftBtUp":
        // Handle left mouse button release
        left_bt_up(appHandle);
        break;

      case "blockRender":
        // Set the time to block rendering (for performance testing)
        renderBlockTime = data.blockTime;
        break;

      case "autoAnimation":
        // Toggle auto-animation
        set_auto_animation(appHandle, data.autoAnimation);
        break;

      default:
        break;
    }
  };

  // Notify the main thread that the worker is ready
  self.postMessage({ ty: "workerIsReady" });
}
init_wasm_in_worker();

function createWorkerAppWindow(offscreenCanvas, devicePixelRatio) {
  // Create the rendering window using the offscreen canvas
  create_window_by_offscreen_canvas(
    appHandle,
    offscreenCanvas,
    devicePixelRatio
  );

  // Check the preparation state
  getPreparationState();

  // Start the frame loop
  requestAnimationFrame(enterFrame);
}

/**
 * Start rendering frames
 *
 * https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/requestAnimationFrame
 * requestAnimationFrame is synchronized with the window's drawing. Manually limiting the frame rate here
 * may cause visual stuttering due to inconsistency with the window refresh rate.
 *
 * TODO: Wait 1 second between the first 3 frames
 */
let frameIndex = 0;
let frameCount = 0;
let frameFlag = 0;

function enterFrame(_dt) {
  if (appHandle === 0 || isStoppedRunning) return;

  // Execute the app's frame loop when it's ready
  if (initFinished > 0) {
    if (
      frameIndex >= frameFlag ||
      (frameIndex < frameFlag && frameCount % 60 == 0)
    ) {
      enter_frame(appHandle);
      frameIndex++;
    }
    frameCount++;
  } else {
    // Check if the app is ready
    getPreparationState();
  }
  requestAnimationFrame(enterFrame);
}

/** Get the Bevy app's readiness state */
function getPreparationState() {
  initFinished = is_preparation_completed(appHandle);
}

/** Send ray pick results to the main thread */
function send_pick_from_worker(pickList) {
  self.postMessage({ ty: "pick", list: pickList });
}

/** Execute a blocking operation (for performance testing) */
function block_from_worker() {
  const start = performance.now();
  while (performance.now() - start < renderBlockTime) { }
}
