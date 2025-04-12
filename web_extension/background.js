console.log("Starting timeline_plugin_web firefox extension")

browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === "reportVisit") {
    fetch(CONFIG.url+"/api/plugin/timeline_plugin_web/register_visit", {method: "POST", body: JSON.stringify(message.data)}).then(async res => {
      await res.text();
    })
  }
})