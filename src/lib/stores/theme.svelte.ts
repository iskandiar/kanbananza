function createThemeStore() {
  let theme = $state<'dark' | 'light'>('dark');

  function init() {
    const saved = localStorage.getItem('theme') as 'dark' | 'light' | null;
    theme = saved ?? 'dark';
    document.documentElement.classList.toggle('light', theme === 'light');
  }

  function toggle() {
    theme = theme === 'dark' ? 'light' : 'dark';
    document.documentElement.classList.toggle('light', theme === 'light');
    localStorage.setItem('theme', theme);
  }

  return {
    get current() { return theme; },
    init,
    toggle
  };
}

export const themeStore = createThemeStore();
