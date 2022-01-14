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
            },
            body: {
                boost: 1
            }
        },
        bool: "OR",
        expand: true
    }

    INDEX.search(box.value, config).forEach(function (result) {
        listItem = document.createElement("li");
        listItem.className = "search-result-item";
        listItem.innerHTML =
            "<a href='" + result.doc.uri + "'>" + result.doc.title +
            "<p class='search-result-item-preview'>" + searchPreview(result.doc.body) + "</p>" +
            "</a>";

        list.appendChild(listItem);
    });
}

function searchPreview(body) {
    return body.substring(0, 100)
        .replace(/=+/g, "")
        .replace(/#+/g, "")
        .replace(/\*+/g, "")
        .replace(/_+/g, "") +
        "...";
}

function disableScrollifMenuOpen() {
    var checkbox = document.getElementById('menu-toggle-switch');

    if (checkbox.checked) {
        document.body.style.overflow = 'hidden';
    } else {
        document.body.style.overflow = 'auto';
    }
}

function atTop() {
    var nav = document.getElementsByClassName("sidebar-right")[0];

    return window.scrollY <= nav.offsetTop + 50;
}

function navTouchingBottom() {
    var nav = document.getElementsByClassName("page-nav")[0];

    var height = Math.max(
        document.body.scrollHeight, document.documentElement.scrollHeight,
        document.body.offsetHeight, document.documentElement.offsetHeight,
        document.body.clientHeight, document.documentElement.clientHeight
    );
    // Magic number determined
    // by height of bottom wave
    return window.scrollY + nav.offsetTop + nav.offsetHeight >= height - 230;
}

function scrolledUp() {
    var height = Math.max(
        document.body.scrollHeight, document.documentElement.scrollHeight,
        document.body.offsetHeight, document.documentElement.offsetHeight,
        document.body.clientHeight, document.documentElement.clientHeight
    );

    // Magic number determined
    // by height of bottom wave
    return window.scrollY + window.innerHeight < height - 230;
}

function dragRightMenu() {
    if (atTop()) {
        document.getElementById('page-nav').classList.remove('fixed');
        document.getElementsByClassName('sidebar-right')[0].classList.remove('bottom');
    } else if (scrolledUp()) {
        document.getElementById('page-nav').classList.add('fixed');
        document.getElementsByClassName('sidebar-right')[0].classList.remove('bottom');
    } else if (navTouchingBottom()) {
        document.getElementById('page-nav').classList.remove('fixed');
        document.getElementsByClassName('sidebar-right')[0].classList.add('bottom');
    } else {
        document.getElementById('page-nav').classList.add('fixed');
        document.getElementsByClassName('sidebar-right')[0].classList.remove('bottom');
    }
}

function isVisible(element) {
    var rect = element.getBoundingClientRect();
    var elemTop = rect.top;
    var elemBottom = rect.bottom;

    var isVisible = (elemTop >= 0) && (elemBottom <= window.innerHeight);
    return isVisible;
}

function toggleColor() {
    var color = localStorage.getItem('doctave-color')

    if (color === 'dark') {
        localStorage.setItem('doctave-color', 'light');
    } else {
        localStorage.setItem('doctave-color', 'dark');
    }

    setColor();
}

function setColor() {
    var color = localStorage.getItem('doctave-color')

    if (color === 'dark') {
        document.querySelector("link[rel='stylesheet'][href*='prism-']").href = BASE_PATH + "assets/prism-atom-dark.css?v=" + DOCTAVE_TIMESTAMP;
        document.getElementsByTagName('html')[0].classList.remove('light');
        document.getElementsByTagName('html')[0].classList.add('dark');
    } else {
        document.querySelector("link[rel='stylesheet'][href*='prism-']").href = BASE_PATH + "assets/prism-ghcolors.css?" + DOCTAVE_TIMESTAMP;
        document.getElementsByTagName('html')[0].classList.remove('dark');
        document.getElementsByTagName('html')[0].classList.add('light');
    }
}

document.getElementById("light-dark-mode-switch").addEventListener("click", toggleColor);


// Initialize mermaid.js based on color theme
var color = localStorage.getItem('doctave-color')
if (color === 'dark') {
    console.log("DARK MODE");
    mermaid.initialize({ 'theme': 'dark' });
} else {
    mermaid.initialize({ 'theme': 'default' });
}

// Setup Katex
var mathElements = document.getElementsByClassName("math");

const macros = {}

for (let element of mathElements) {
    let latex = element.textContent;

    try {
        katex.render(latex, element, {
            displayMode: true,
            macros: macros,
        });
    } catch (e) {
        if (e instanceof katex.ParseError) {
            // KaTeX can't parse the expression
            var error_message = e.message
                .replaceAll(/^KaTeX parse error: /g, "Error parsing math notation:\n")
                .replaceAll(/&/g, "&amp;")
                .replaceAll(/</g, "&lt;")
                .replaceAll(/>/g, "&gt;")
                .replaceAll("\n", "<br />");

            element.innerHTML = "<p class='katex-error-msg'>" + error_message + "</p>" + latex.trim().replaceAll("\n", "<br />");
            element.classList.add("katex-error");
        } else {
            throw e;  // other error
        }
    }
}

// Setup Prism
Prism.plugins.autoloader.languages_path = BASE_PATH + 'assets/prism-grammars/';


// Load search index
var INDEX;

fetch(BASE_PATH + 'search_index.json')
    .then(function (response) {
        if (!response.ok) {
            throw new Error("HTTP error " + response.status);
        }
        return response.json();
    })
    .then(function (json) {
        INDEX = elasticlunr.Index.load(json)
        document.getElementById('search-box').oninput = search;
        search();
    });

// Setup keyboard shortcuts
document.onkeydown = function (e) {
    var searchResults = document.getElementById('search-results');
    var first = searchResults.firstChild;
    var searchBox = document.getElementById('search-box');

    switch (e.keyCode) {
        case 83: // The S key
            if (document.activeElement == searchBox) {
                break;
            } else {
                searchBox.focus();
                e.preventDefault();
            }
            break;
        case 38: // if the UP key is pressed
            if (document.activeElement == (searchBox || first)) {
                break;
            } else {
                document.activeElement.parentNode.previousSibling.firstChild.focus();
                e.preventDefault();
            }
            break;
        case 40: // if the DOWN key is pressed
            if (document.activeElement == searchBox) {
                first.firstChild.focus();
                e.preventDefault();
            } else {
                document.activeElement.parentNode.nextSibling.firstChild.focus();
                e.preventDefault();
            }
            break;
        case 27: // if the ESC key is pressed
            if (first) {
                searchResults.innerHTML = '';
            }
            break;
    }
}

disableScrollifMenuOpen();
dragRightMenu();
setColor();
