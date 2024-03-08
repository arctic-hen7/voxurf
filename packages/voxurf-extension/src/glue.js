export function attach_debugger(tabId) {
  return new Promise((resolve, reject) => {
    chrome.debugger.attach({ tabId }, "1.2", () => {
      resolve();
    })
  })
}

export function detach_debugger(tabId) {
  return new Promise((resolve, reject) => {
    chrome.debugger.detach({ tabId }, () => resolve());
  });
}

export function get_raw_ax_tree(tabId) {
  return new Promise((resolve, reject) => {
    chrome.debugger.sendCommand(
      { tabId },
      "Accessibility.getFullAXTree",
      {},
      nodes => resolve(nodes)
    );
  });
}

export function get_tab_id() {
  return new Promise((resolve, reject) => {
    chrome.tabs.query({ active: true, currentWindow: true }, tabs => {
      resolve(tabs[0].id);
    });
  });
}

export function click_element(tabId, selector) {
  return new Promise((resolve, reject) => {
    chrome.scripting.executeScript({
      target: { tabId },
      func: host_click,
      args: [ selector ],
    }, () => resolve())
  })
}
export function fill_element(tabId, selector, text) {
  return new Promise((resolve, reject) => {
    chrome.scripting.executeScript({
      target: { tabId },
      func: host_fill,
      args: [ selector, text ],
    }, () => resolve())
  })
}

// --- These will be executed in the host ---
function host_click(selector) {
  document.querySelector(selector).click()
}
function host_fill(selector, text) {
  document.querySelector(selector).value = text;
}
// --- End functions to be executed in the host ---

export function dom_enable(tabId) {
  return new Promise((resolve, reject) => {
    chrome.debugger.sendCommand(
      { tabId },
      "DOM.enable",
      {},
      () => resolve()
    );
  });
}

export function dom_disable(tabId) {
  return new Promise((resolve, reject) => {
    chrome.debugger.sendCommand(
      { tabId },
      "DOM.disable",
      {},
      () => resolve()
    );
  });
}

export function dom_id_to_selector(id, tabId) {
  return new Promise((resolve, reject) => {
    // We need to get the document to force setting `nodeId`s,
    // see https://issues.chromium.org/issues/41487727
    chrome.debugger.sendCommand(
      { tabId },
      "DOM.resolveNode",
      { backendNodeId: id },
      (res) => {
        chrome.debugger.sendCommand(
          { tabId },
          "DOM.getDocument",
          {},
          () => {
            chrome.debugger.sendCommand(
              { tabId },
              "DOM.requestNode",
              // We get data from throughout the tree, so this shoudl traverse everything
              // (including shadow roots and iframes)
              { objectId: res.object.objectId },
              ({ nodeId }) => {
                chrome.debugger.sendCommand(
                  { tabId },
                  "DOM.setAttributeValue",
                  { nodeId, name: "data-voxurf-id", value: `${id}` },
                  () => resolve(`[data-voxurf-id='${id}']`)
                );
              }
            );
          }
        );
      }
    );
  });
}

export function execute_js(tabId, script) {
  return new Promise((resolve, reject) => {
    chrome.debugger.sendCommand(
      { tabId },
      "Runtime.evaluate",
      { expression: script, userGesture: true },
      () => resolve()
    )
  });
}
