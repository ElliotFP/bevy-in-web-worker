/**
 * To avoid the browser downloading the same Wasm file simultaneously,
 * the worker loading is executed first.
 * After the engine instance is successfully created in the worker,
 * the main thread's engine instance is loaded using the browser cache.
 */

const worker = new Worker("./worker.js");

// Check if the worker is ready
let workerIsReady = false;

// Latest pick result
let latestPick = [];

// Listen for messages from the worker
worker.onmessage = async (event) => {
  let data = event.data;
  window.blockMS(window.onmessageBlockTime);

  switch (data.ty) {
    case "workerIsReady":
      workerIsReady = true;
      // Start loading the main thread instance
      loadMainThreadEngine();
      addMouseEventObserver();
      break;

    case "pick":
      // Other business logic dependent on pick
      // ...

      // Display pick results on the webpage
      let ele = document.getElementById("pick-list");
      ele.innerText = data.list;

      latestPick = data.list;
      // Notify the worker which entities to enable hover effect for
      worker.postMessage({ ty: "hover", list: latestPick });
      break;

    default:
      break;
  }
};

// Create app window
function createWorkerAppWindow() {
  delayExecute(() => {
    if (workerIsReady) {
      let loading = document.getElementById("loading");
      loading.style.display = "none";

      // Create rendering window
      let canvas = document.getElementById("worker-thread-canvas");
      let offscreenCanvas = canvas.transferControlToOffscreen();
      let devicePixelRatio = window.devicePixelRatio;
      worker.postMessage(
        { ty: "init", canvas: offscreenCanvas, devicePixelRatio },
        [offscreenCanvas]
      );

      return true;
    }
    return false;
  });
}

function delayExecute(fn, delay = 50) {
  function execute() {
    if (fn()) {
      clearInterval(timer);
    }
  }
  const timer = setInterval(execute, delay);
}

// Launch the App
function launch() {
  // First check the browser environment
  if ("navigator" in window && "gpu" in navigator) {
    navigator.gpu
      .requestAdapter()
      .then((_adapter) => {
        // Adjust canvas style
        resizeCanvasBy("main-thread-container");
        resizeCanvasBy("worker-thread-container");

        // Browser supports WebGPU
        createWorkerAppWindow();
      })
      .catch((_error) => {
        console.error(_error);
        showAlert();
      });
  } else {
    // Browser doesn't support navigator.gpu
    showAlert();
  }
}
launch();

function showAlert() {
  alert("Please use Chrome or Edge 113+ browser version");
}

// Set canvas based on parent container
function resizeCanvasBy(containerID) {
  let elem = document.getElementById(containerID);
  let canvas = elem.children[0];
  let ratio = window.devicePixelRatio;
  canvas.width = elem.clientWidth * ratio;
  canvas.height = elem.clientHeight * ratio;
  canvas.style.width = elem.clientWidth + "px";
  canvas.style.height = elem.clientHeight + "px";
  canvas.style.maxWidth = elem.clientWidth + "px";
  canvas.style.maxHeight = elem.clientHeight + "px";
}

// Add mouse event listener
function addMouseEventObserver() {
  // Event listener for worker thread engine instance
  let workerContainer = document.getElementById("worker-thread-container");
  workerContainer.addEventListener("mousemove", function (event) {
    window.blockMS(window.mousemoveBlockTime);
    // Clear the previous pick cache before sending mouse move event to worker
    latestPick = [];
    worker.postMessage({ ty: "mousemove", x: event.offsetX, y: event.offsetY });
  });

  workerContainer.addEventListener("mousedown", function (event) {
    if (typeof latestPick[0] !== "undefined") {
      worker.postMessage({
        ty: "leftBtDown",
        pickItem: latestPick[0],
        x: event.offsetX,
        y: event.offsetY,
      });
    }
  });

  workerContainer.addEventListener("mouseup", function (_event) {
    worker.postMessage({ ty: "leftBtUp" });
  });

  workerContainer.addEventListener("click", function (event) {
    if (Array.isArray(latestPick) && latestPick.length > 0) {
      worker.postMessage({
        ty: "select",
        list: latestPick,
      });
    }
  });

  // Event listener for main thread engine instance
  let mainContainer = document.getElementById("main-thread-container");
  mainContainer.addEventListener("mousemove", function (event) {
    window.blockMS(window.mousemoveBlockTime);
    // Clear the previous pick cache
    latestPick = [];
    window.mouse_move(event.offsetX, event.offsetY);
  });

  mainContainer.addEventListener("mousedown", function (event) {
    if (typeof latestPick[0] !== "undefined") {
      window.left_bt_down(latestPick[0], event.offsetX, event.offsetY);
    }
  });

  mainContainer.addEventListener("mouseup", function (_event) {
    window.left_bt_up();
  });

  mainContainer.addEventListener("click", function (_event) {
    window.set_selection(latestPick);
  });
}

// Load the main thread engine instance
function loadMainThreadEngine() {
  var script = document.createElement("script");
  script.type = "module";
  script.src = "./main_thread_engine_instance.js";
  document.body.appendChild(script);
}

/** Ray pick result from the main thread engine instance */
function send_pick_from_rust(pickList) {
  // Display pick results on the webpage
  let ele = document.getElementById("pick-list");
  ele.innerText = pickList;
  latestPick = pickList;

  window.set_hover(latestPick);
}
window.send_pick_from_rust = send_pick_from_rust;

function block_from_rust() {
  window.blockMS(window.renderBlockTime);
}
window.block_from_rust = block_from_rust;

window.blockWorkerRender = (dt) => {
  worker.postMessage({ ty: "blockRender", blockTime: dt });
};

// Start engine instance
window.start_worker_app = () => {
  worker.postMessage({ ty: "startRunning" });
  setContainerOpacity("100%");
};

// Stop engine instance
window.stop_worker_app = () => {
  worker.postMessage({ ty: "stopRunning" });
  setContainerOpacity("50%");
};

// Turn on/off engine animation
window.set_worker_auto_animation = (needsAnimation) => {
  worker.postMessage({ ty: "autoAnimation", autoAnimation: needsAnimation });
};

function setContainerOpacity(opacity) {
  let ele = document.getElementById("worker-thread-container");
  ele.style.opacity = opacity;
}
