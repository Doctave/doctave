function search() {
    box = document.getElementById('search-box');
    list = document.getElementById('search-results');
    list.innerHTML = '';

    if (box.value == "") {
        return
    }

    config = {
        fields: {
            title: {
                boost: 2,
                bool: "AND"
            },
            body: {
                boost: 1
            }
        },
        bool: "OR",
        expand: true
    }

    INDEX.search(box.value, config).forEach(function(result) {
        listItem = document.createElement("li");
        listItem.className = "search-result-item";
        listItem.innerHTML = "<a href='" + result.doc.uri + "'>" + result.doc.title + "</a>";

        list.appendChild(listItem);
    });
}

function scrollTop() {
    var supportPageOffset = window.pageXOffset !== undefined;
    var isCSS1Compat = ((document.compatMode || "") === "CSS1Compat");

    return supportPageOffset ? window.pageYOffset : isCSS1Compat ? document.documentElement.scrollTop : document.body.scrollTop;
}

// Checks if the search bar is visible, as a proxy for
// determining when we should start scrolling.
function shouldDragPageNav() {
    var search = document.getElementById('search-form');

    return !isVisible(search);
}

function isVisible(element) {
    var rect = element.getBoundingClientRect();
    var elemTop = rect.top;
    var elemBottom = rect.bottom;

    var isVisible = (elemTop >= 0) && (elemBottom <= window.innerHeight);
    return isVisible;
}

function dragPageNav() {
    if (shouldDragPageNav()) {
        pageNav = document.getElementById('page-nav');
        pageNav.style.position = 'fixed';
        pageNav.style.top = '13.5rem';
    } else {
        pageNav = document.getElementById('page-nav');
        pageNav.style.position = '';
        pageNav.style.top = '';
    }
}

// Page nav listeners
window.addEventListener('scroll', function(e) {
    dragPageNav();
});
// Don't reset scrolling on livereload
window.addEventListener('scroll', function() {
    localStorage.setItem('scrollPosition', window.scrollY);
}, false);

window.addEventListener('load', function() {
    if (localStorage.getItem('scrollPosition') !== null)
        window.scrollTo(0, localStorage.getItem('scrollPosition'));
}, false);


// Initialize mermaid JS
mermaid.initialize({
    startOnLoad: true
});

var INDEX;

// Load search index
fetch('/search_index.json')
    .then(function(response) {
        if (!response.ok) {
            throw new Error("HTTP error " + response.status);
        }
        return response.json();
    })
    .then(function(json) {
        INDEX = elasticlunr.Index.load(json)
        document.getElementById('search-box').oninput = search;
        search();
    });

dragPageNav();
