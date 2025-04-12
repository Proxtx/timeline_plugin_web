browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === "reportVisit") {
    fetch("http://localhost:8001/api/plugin/timeline_plugin_web/register_visit", {method: "POST", body: JSON.stringify(message.data)}).then(async res => {
      await res.text();
    })
  }
})