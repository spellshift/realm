

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
      
    console.log(Date.now())

    var cssFile = document.querySelector('[rel="stylesheet"]');

    if (sessionStorage.getItem("darkmode") == 'true') {
        cssFile.setAttribute('href', '{{ "assets/css/just-the-docs-" | relative_url }}' + 'dark' + '.css');
        toggleDarkMode.textContent = 'Light mode';
        console.log("dark " + Date.now())
    } else {
        jtd.setTheme('light');
        cssFile.setAttribute('href', '{{ "assets/css/just-the-docs-" | relative_url }}' + 'light' + '.css');
        console.log("light " + Date.now())
    }
    console.log(jtd.getTheme());
}
  