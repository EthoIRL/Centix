function updateQueryURL(query, window) {
    let url = window.location.href;

    if (url.includes("&" + query + "=true") || url.includes("&" + query + "=false")
        || url.includes("?" + query + "=true") || url.includes("?" + query + "=false")) {
        url = url.replace("&" + query + "=true", '');
        url = url.replace("?" + query + "=true", '');
    } else {
        let addURL = ''
        if (url.indexOf('?') > -1) {
            addURL += '&' + query + '=true'
        } else {
            addURL += '?' + query + '=true'
        }
        url += addURL;
    }

    if(url.indexOf('?') <= 0) {
        url = url.replace("&", '?');
    }

    window.location.href = url;
}

function clearQueries(window) {
    window.history.pushState({}, document.title, window.location.pathname);
}

function clearSortingParams(window) {
    clearQueries(window);
    const queryContainer = document.getElementById("post-sort-options");

    for (const child of queryContainer.children) {
        child.classList.remove("sort-selected");
    }

    const defaultSort = document.getElementById("default-sort");
    defaultSort.classList.add("sort-item");

    window.location.reload();
}