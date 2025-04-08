// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="configuration.html"><strong aria-hidden="true">2.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="recipes.html"><strong aria-hidden="true">3.</strong> Recipes</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="metadata.html"><strong aria-hidden="true">3.1.</strong> Metadata</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="rpm.html"><strong aria-hidden="true">3.1.1.</strong> RPM</a></li><li class="chapter-item expanded "><a href="deb.html"><strong aria-hidden="true">3.1.2.</strong> DEB</a></li><li class="chapter-item expanded "><a href="pkg.html"><strong aria-hidden="true">3.1.3.</strong> PKG</a></li><li class="chapter-item expanded "><a href="apk.html"><strong aria-hidden="true">3.1.4.</strong> APK</a></li></ol></li><li class="chapter-item expanded "><a href="scripts.html"><strong aria-hidden="true">3.2.</strong> Scripts</a></li><li class="chapter-item expanded "><a href="env.html"><strong aria-hidden="true">3.3.</strong> Env</a></li><li class="chapter-item expanded "><a href="inheritance.html"><strong aria-hidden="true">3.4.</strong> Inheritance</a></li></ol></li><li class="chapter-item expanded "><a href="images.html"><strong aria-hidden="true">4.</strong> Images</a></li><li class="chapter-item expanded "><a href="usage.html"><strong aria-hidden="true">5.</strong> Build a package</a></li><li class="chapter-item expanded "><a href="signing.html"><strong aria-hidden="true">6.</strong> Signing packages</a></li><li class="chapter-item expanded "><a href="output.html"><strong aria-hidden="true">7.</strong> Formatting output</a></li><li class="chapter-item expanded "><a href="new.html"><strong aria-hidden="true">8.</strong> Create new recipes and images</a></li><li class="chapter-item expanded "><a href="edit.html"><strong aria-hidden="true">9.</strong> Edit recipes, images and config</a></li><li class="chapter-item expanded "><a href="completions.html"><strong aria-hidden="true">10.</strong> Shell completions</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
