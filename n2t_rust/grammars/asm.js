module.exports = grammar({
  name: 'asm',

  rules: {
    source_file: $ => $.asm,

    asm: $ => seq(
      repeat($.intermediate_instruction),
      optional($.instruction)
    ),

    instruction: $ => choice(
      $.label,
      $.a_instruction,
      $.c_instruction
    ),

    intermediate_instruction: $ => seq(
      $.instruction,
      repeat1(/\s/)
    ),

    label: $ => seq(
      '(',
      $.identifier,
      ')'
    ),

    a_instruction: $ => seq(
      '@',
      choice($.identifier, $.number)
    ),

    c_instruction: $ => seq(
      optional($.assign),
      $.op,
      optional($.jmp)
    ),

    assign_char: $ => choice('A', 'M', 'D'),
    
    op_char: $ => choice(
      $.assign_char,
      '0', '1', '!', '-', '+', '|', '&'
    ),

    assign: $ => seq(
      repeat1($.assign_char),
      '='
    ),

    op: $ => repeat1($.op_char),

    jmp: $ => seq(
      ';',
      choice('JGT', 'JEQ', 'JGE', 'JLT', 'JNE', 'JLE', 'JMP')
    ),

    identifier: $ => /[a-zA-Z_.:\$][a-zA-Z0-9_.:\$]*/,
    
    number: $ => /\d+/,

    comment: $ => token(seq('//', /.*/)),
  },

  extras: $ => [
    /\s/,
    $.comment,
  ],
});