

window.onload=function(){
    const toggleDarkMode = document.querySelector('.js-toggle-dark-mode');

    window.jtd.addEvent(toggleDarkMode, 'click', function(){
      // Switch theme on click
      if (jtd.getTheme() === 'dark') {
        jtd.setTheme('light');
        toggleDarkMode.textContent = 'Dark mode';
        sessionStorage.setItem("darkmode", false)
      } else {
        jtd.setTheme('dark');
        toggleDarkMode.textContent = 'Light mode';
        sessionStorage.setItem("darkmode", true)
      }
    })
      
    var cssFile = document.querySelector('[rel="stylesheet"]');

    if (sessionStorage.getItem("darkmode") == 'true') {
        toggleDarkMode.textContent = 'Light mode';
    } else {
        toggleDarkMode.textContent = 'Dark mode';
    }
}
  