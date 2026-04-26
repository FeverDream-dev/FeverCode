// FeverCode — Site JS
(function () {
  var toggle = document.getElementById('nav-toggle');
  var links = document.getElementById('nav-links');

  if (toggle && links) {
    toggle.addEventListener('click', function () {
      links.classList.toggle('open');
    });

    links.addEventListener('click', function (e) {
      if (e.target.tagName === 'A') {
        links.classList.remove('open');
      }
    });
  }

  function initTabs() {
    var tabContainer = document.querySelector('.install-tabs');
    if (!tabContainer) return;

    var tabs = tabContainer.querySelectorAll('.install-tab');
    var panels = document.querySelectorAll('.install-tab-panel');

    tabs.forEach(function (tab) {
      tab.addEventListener('click', function () {
        var target = this.getAttribute('data-tab');

        tabs.forEach(function (t) {
          t.classList.remove('active');
          t.setAttribute('aria-selected', 'false');
        });

        panels.forEach(function (p) {
          p.classList.remove('active');
        });

        this.classList.add('active');
        this.setAttribute('aria-selected', 'true');

        var panel = document.getElementById('tab-' + target);
        if (panel) panel.classList.add('active');
      });
    });
  }

  function initCopyButtons() {
    var buttons = document.querySelectorAll('.copy-btn');

    buttons.forEach(function (btn) {
      btn.addEventListener('click', function () {
        var targetId = this.getAttribute('data-copy');
        var el = document.getElementById(targetId);
        if (!el) return;

        var text = el.textContent || el.innerText;
        text = text.replace(/^\s*\n/, '').trim();

        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(text).then(function () {
            showCopied(btn);
          });
        } else {
          var ta = document.createElement('textarea');
          ta.value = text;
          ta.style.position = 'fixed';
          ta.style.opacity = '0';
          document.body.appendChild(ta);
          ta.select();
          try {
            document.execCommand('copy');
            showCopied(btn);
          } catch (e) { /* noop */ }
          document.body.removeChild(ta);
        }
      });
    });
  }

  function showCopied(btn) {
    var orig = btn.textContent;
    btn.textContent = 'Copied!';
    btn.classList.add('copied');
    setTimeout(function () {
      btn.textContent = orig;
      btn.classList.remove('copied');
    }, 2000);
  }

  initTabs();
  initCopyButtons();
})();
