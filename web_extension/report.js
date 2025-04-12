function reportHref (){
  browser.runtime.sendMessage({
    type: "reportVisit",
    data: {client: "Firefox Desktop", website: window.location.href}
  })
}

let lastHref = null;
(async () => {
  while(true) {
    if(window.location.href != lastHref) {
      reportHref();
    }
    lastHref = window.location.href;
    await new Promise(r => setTimeout(r, 1000))
  }
})()

