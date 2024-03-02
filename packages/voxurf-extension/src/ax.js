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

// TODO Make this work
export function execute_js(tabId, script) {
  return new Promise((resolve, reject) => {
    chrome.scripting.executeScript({
      target: { tabId },
      func: fauxval,
      args: [ script ]
    }, () => resolve());
  });
}
