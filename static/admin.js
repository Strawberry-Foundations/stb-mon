async function onDelete(id) {
    if (!confirm("Delete this monitor?")) {
        return
    }

    let res = await fetch(`/api/monitors/${id}`, { method: "DELETE" });

    if (res.status != 200) {
        alert(await res.text());
        return;
    };

    alert("Monitor was deleted");
}

async function onToggle(id) {
    let res = await fetch(`/api/monitors/${id}/toggle`, { method: "PUT" });
    alert(await res.text())
}