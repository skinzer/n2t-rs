module.exports = grammar({
  name: 'hdl',

  rules: {
    source_file: $ => $.chip,

    chip: $ => seq(
      'CHIP',
      $.identifier,
      '{',
      $.chip_body,
      '}'
    ),

    chip_body: $ => seq(
      optional($.in_list),
      optional($.out_list),
      $.part_list,
      optional($.clocked_list)
    ),

    in_list: $ => seq(
      'IN',
      $.pin_list,
      ';'
    ),

    out_list: $ => seq(
      'OUT',
      $.pin_list,
      ';'
    ),

    pin_list: $ => seq(
      $.pin_decl,
      repeat(seq(',', $.pin_decl))
    ),

    pin_decl: $ => seq(
      $.identifier,
      optional($.pin_width)
    ),

    pin_width: $ => seq(
      '[',
      $.number,
      ']'
    ),

    part_list: $ => choice(
      $.builtin_part,
      $.parts
    ),

    builtin_part: $ => seq(
      'BUILTIN',
      ';'
    ),

    parts: $ => seq(
      'PARTS:',
      repeat($.part)
    ),

    part: $ => seq(
      $.identifier,
      '(',
      $.wires,
      ')',
      ';'
    ),

    wires: $ => seq(
      $.wire,
      repeat(seq(',', $.wire))
    ),

    wire: $ => seq(
      $.wire_side,
      '=',
      choice($.wire_side, $.boolean)
    ),

    wire_side: $ => seq(
      $.identifier,
      optional($.sub_bus)
    ),

    sub_bus: $ => seq(
      '[',
      $.number,
      optional(seq('..', $.number)),
      ']'
    ),

    clocked_list: $ => seq(
      'CLOCKED',
      $.simple_pin_list,
      ';'
    ),

    simple_pin_list: $ => seq(
      $.identifier,
      repeat(seq(',', $.identifier))
    ),

    identifier: $ => /[a-zA-Z][a-zA-Z0-9_]*/,
    
    number: $ => /\d+/,
    
    boolean: $ => choice('true', 'false'),

    comment: $ => token(choice(
      seq('//', /.*/),
      seq(
        '/*',
        /[^*]*\*+([^/*][^*]*\*+)*/,
        '/'
      )
    )),
  },

  extras: $ => [
    /\s/,
    $.comment,
  ],
});