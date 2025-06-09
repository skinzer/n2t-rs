module.exports = grammar({
  name: 'vm',

  rules: {
    source_file: $ => $.vm,

    vm: $ => seq(
      repeat($._newline),
      repeat($.vm_instruction_line),
      optional($.vm_instruction)
    ),

    vm_instruction_line: $ => seq(
      $.vm_instruction,
      repeat1($._newline)
    ),

    vm_instruction: $ => choice(
      $.stack_instruction,
      $.op_instruction,
      $.function_instruction,
      $.call_instruction,
      $.return_instruction,
      $.goto_instruction,
      $.label_instruction
    ),

    stack_instruction: $ => seq(
      choice('push', 'pop'),
      $.memory_segment,
      $.number
    ),

    op_instruction: $ => choice(
      'add', 'sub', 'neg', 'lt', 'gt', 'eq', 'and', 'or', 'not'
    ),

    function_instruction: $ => seq(
      'function',
      $.identifier,
      $.number
    ),

    call_instruction: $ => seq(
      'call',
      $.identifier,
      $.number
    ),

    return_instruction: $ => 'return',

    label_instruction: $ => seq(
      'label',
      $.identifier
    ),

    goto_instruction: $ => seq(
      choice('goto', 'if-goto'),
      $.identifier
    ),

    memory_segment: $ => choice(
      'argument',
      'local',
      'static',
      'constant',
      'this',
      'that',
      'pointer',
      'temp'
    ),

    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_.]*/,
    
    number: $ => /\d+/,

    _newline: $ => '\n',

    comment: $ => token(seq('//', /.*/)),
  },

  extras: $ => [
    /[ \t]/,
    $.comment,
  ],
});