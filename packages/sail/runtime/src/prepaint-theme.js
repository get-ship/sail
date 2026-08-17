try {
  var _d = document.documentElement;
  var _t = null;
  try { _t = localStorage.getItem('ship-site-theme'); } catch (_e) {}
  if (!_t) {
    var _mt = document.querySelector('meta[name="sail-default-theme"]');
    _t = (_mt && _mt.getAttribute('content')) || 'sail';
  }
  _d.setAttribute('theme', _t);
  _d.removeAttribute('data-theme');
  _d.removeAttribute('data-theme-preview');
  var _m = null;
  try { _m = localStorage.getItem('ship-site-theme-mode'); } catch (_e) {}
  if (!_m) {
    var _mm = document.querySelector('meta[name="sail-default-mode"]');
    _m = (_mm && _mm.getAttribute('content')) || 'light';
  }
  var _paint = 'light';
  if (_m === 'dark') _paint = 'dark';
  else if (_m === 'light') _paint = 'light';
  else if (_m === 'system') {
    _paint = (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches)
      ? 'dark' : 'light';
  }
  _d.setAttribute('mode', _paint);
  _d.removeAttribute('data-mode');
  if (_d.classList) _d.classList.remove('dark');
} catch (_e) {}
