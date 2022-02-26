(function($) {
    "use strict";

    /* ==============================================
    ANIMATION -->
    =============================================== */

    new WOW({
        boxClass: 'wow', // default
        animateClass: 'animated', // default
        offset: 0, // default
        mobile: true, // default
        live: true // default
    }).init();

    /* ==============================================
    LIGHTBOX -->
    =============================================== */

    jQuery('a[data-gal]').each(function() {
        jQuery(this).attr('rel', jQuery(this).data('gal'));
    });
    jQuery("a[data-rel^='prettyPhoto']").prettyPhoto({
        animationSpeed: 'slow',
        theme: 'light_square',
        slideshow: true,
        overlay_gallery: true,
        social_tools: false,
        deeplinking: false
    });

    /* ==============================================
    SCROLL -->
    =============================================== */

    $(function() {
        $('a[href*=#]:not([href=#])').click(function(e) {
            $('html,body').animate({
                scrollTop: $(this.hash).offset().top - 5
            }, 1000);
        });
    });

    /* ==============================================
    SCROLLSPY -->
    =============================================== */

    $('body').scrollspy({
        target: '.page-sidebar'
    });

    $('[data-spy="scroll"]').each(function () {
      var $spy = $(this).scrollspy('refresh')
    })

    /* ==============================================
    VIDEO FIX -->
    =============================================== */

    $(document).ready(function() {
        // Target your .container, .wrapper, .post, etc.
        $(".media").fitVids();
    });

    /* ==============================================
    VIDEO FIX -->
    =============================================== */

    $('.page-sidebar>nav>li>a').click(function(e) {
        $('.page-sidebar>nav>li').removeClass('active');
        $(this).parent().addClass('active');
    });
})(jQuery);
