'use strict';

$(function() {
  function animateCSS(node, animationName, callback) {
      node.classList.add('animated', animationName)

      function handleAnimationEnd() {
          node.classList.remove('animated', animationName)
          node.removeEventListener('animationend', handleAnimationEnd)

          if (typeof callback === 'function') callback()
      }

      node.addEventListener('animationend', handleAnimationEnd)
  }

  $('tr.table-danger').click(function(obj) {
    var name = obj.currentTarget.children[0].innerText

    $('h2#selected-check').first().removeClass('d-none');
    $('h2#selected-check')[0].innerText = name;

    const tableID = name.toLowerCase().replace(/ /g, '_');
    const tableSelector = 'table#' + tableID;

    // Hide all the tables
    $('table.table.table-striped').each(function(index, elem) {
      $('table#' + elem.id).addClass('d-none');
    });

    // Show the selected table
    $(tableSelector).first().removeClass('d-none');

    animateCSS(obj.currentTarget, 'pulse', function() {
      // Scroll to the table
      $('html, body').animate({
        scrollTop: $('#selected-check').offset().top,
      }, 800);
    });
  });
});


