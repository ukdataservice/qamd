'use strict';

$(function() {

  $('tr.table-danger').click(function(obj) {
    var name = obj.currentTarget.children[0].innerText

    $('h2#selected-check').first().removeClass('d-none');
    $('h2#selected-check')[0].innerText = name;

    var selector = 'table#' + name.toLowerCase().replace(/ /g, '_');

    // hide all the tables
    $('table.table.table-striped').each(function(index, elem) {
      $('table#' + elem.id).addClass('d-none');
    });

    // show the selected table
    $(selector).first().removeClass('d-none');
  });
});

