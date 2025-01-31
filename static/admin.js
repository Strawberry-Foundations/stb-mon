function elem(el) {
    return document.getElementById(el);
}

function uriEnc(text) {
    return encodeURIComponent(text);
}

async function onDelete(id) {
    if (!confirm("Delete this monitor?")) {
        return;
    }

    let res = await fetch(`/api/monitors/${id}`, { method: "DELETE" });

    if (res.status != 200) {
        alert(await res.text());
        return;
    };

    alert("Monitor was deleted");
    window.location.reload();
}

async function onToggle(id) {
    let res = await fetch(`/api/monitors/${id}/toggle`, { method: "PATCH" });
    alert(await res.text())
    window.location.reload();
}

function onAddTypeChange() {
    let serviceType = elem("service-type").value;

    let tcpOptions = elem("tcp-options");
    let httpOptions = elem("http-options");

    tcpOptions.hidden = serviceType != "tcp";
    httpOptions.hidden = serviceType != "http";
}

function onHttpExpectedResponseChange() {
    let responseType = elem("http-expected-response");

    let statusCodeOptions = elem("http-sc-options");
    let responseBodyOptions = elem("http-response-options");

    statusCodeOptions.hidden = responseType.value == "any";
    responseBodyOptions.hidden = responseType.value != "res";
}

async function onAdd() {
    let serviceType = elem("service-type").value;
    let serviceName = elem("service-name").value;
    let intervalMins = elem("interval").value;
    let timeoutSecs = elem("timeout").value;

    let url = `/api/monitors?ty=${serviceType}&na=${uriEnc(serviceName)}&in=${intervalMins}&to=${timeoutSecs}`;

    switch (serviceType) {
        case "tcp": {
            let socketAddr = elem("sock-addr").value;
            let expectedResponse = elem("tcp-expected-response").value;
            
            url += `&sa=${uriEnc(socketAddr)}&exre=${expectedResponse}`;
            // TODO: add sh and ex
            let res = await fetch(url, { method: "PUT" });
            alert(await res.text());
            if (res.status == 201) {
                document.location.reload();
            };

            break;
        }
        case "http": {
            let method = elem("method").value;
            let serviceUrl = elem("url").value;
            let headers = elem("headers").value.trim();
            let expectedResponse = elem("http-expected-response").value;
            let requestBody = elem("request-body").value;
            
            url += `&url=${uriEnc(serviceUrl)}&hds=${uriEnc(headers)}&exre=${expectedResponse}&body=${btoa(requestBody)}`

            switch (expectedResponse) {
                case "any": {
                    let res = await fetch(url, { method: "PUT" });
                    alert(await res.text());
                    if (res.status == 201) {
                        document.location.reload();
                    };

                    break;
                }
                case "sc": {
                    let statusCode = elem("status-code").value;
                    
                    url += `&co=${statusCode}`;

                    let res = await fetch(url, { method: "PUT" });
                    alert(await res.text());
                    if (res.status == 201) {
                        document.location.reload();
                    };
                    
                    break;
                }
                case "res": {
                    // TODO
                    break;
                }
            }

            break;
        }
        default: console.log("unknown service type " + serviceType);
    }
}