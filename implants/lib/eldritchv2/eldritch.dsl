File = {Statement | newline} eof .

Statement = DefStmt | IfStmt | ForStmt | SimpleStmt .

DefStmt = 'def' identifier '(' [Parameters [',']] ')' ':' Suite .

Parameters = Parameter {',' Parameter}.

Parameter  = identifier
           | identifier '=' Expression
           | '*'
           | '*' identifier
           | '**' identifier
           .

IfStmt = 'if' Expression ':' Suite {'elif' Expression ':' Suite} ['else' ':' Suite] .

ForStmt = 'for' LoopVariables 'in' Expressions ':' Suite .

Suite = [newline indent {Statement} outdent] | SimpleStmt .

SimpleStmt = SmallStmt {';' SmallStmt} [';'] '\n' .
# NOTE: '\n' optional at EOF

SmallStmt = ReturnStmt
          | BreakStmt | ContinueStmt | PassStmt
          | AssignStmt
          | ExprStmt
          | LoadStmt
          .

ReturnStmt   = 'return' [Expressions] .
BreakStmt    = 'break' .
ContinueStmt = 'continue' .
PassStmt     = 'pass' .
AssignStmt   = Expressions ('=' | '+=' | '-=' | '*=' | '/=' | '//=' | '%=' | '&=' | '|=' | '^=' | '<<=' | '>>=') Expressions .
ExprStmt     = Expressions .

LoadStmt = 'load' '(' string {',' [identifier '='] string} [','] ')' .

Expression = IfExpr | PrimaryExpr | UnaryExpr | BinaryExpr | LambdaExpr .

IfExpr = Expression 'if' Expression 'else' Expression .

PrimaryExpr = Operand
            | PrimaryExpr DotSuffix
            | PrimaryExpr CallSuffix
            | PrimaryExpr SubscriptSuffix
            .

Operand = identifier
        | int | float | string | bytes
        | ListExpr | ListComp
        | DictExpr | DictComp
        | '(' [Expressions [',']] ')'
        .

DotSuffix   = '.' identifier .
SubscriptSuffix = '[' [Expressions] [':' Expression [':' Expression]] ']'
                | '[' Expressions ']'
                .
CallSuffix  = '(' [Arguments [',']] ')' .

Arguments = Argument {',' Argument} .
Argument  = Expression | identifier '=' Expression | '*' Expression | '**' Expression .

ListExpr = '[' [Expressions [',']] ']' .
ListComp = '[' Expression {CompClause} ']'.

DictExpr = '{' [Entries [',']] '}' .
DictComp = '{' Entry {CompClause} '}' .
Entries  = Entry {',' Entry} .
Entry    = Expression ':' Expression .

CompClause = 'for' LoopVariables 'in' Expression | 'if' Expression .

UnaryExpr = '+' Expression
          | '-' Expression
          | '~' Expression
          | 'not' Expression
          .

BinaryExpr = Expression {Binop Expression} .

Binop = 'or'
      | 'and'
      | '==' | '!=' | '<' | '>' | '<=' | '>=' | 'in' | 'not' 'in'
      | '|'
      | '^'
      | '&'
      | '<<' | '>>'
      | '-' | '+'
      | '*' | '%' | '/' | '//'
      .

LambdaExpr = 'lambda' [Parameters] ':' Expression .

Expressions = Expression {',' Expression} .
# NOTE: trailing comma permitted only when within [...] or (...).

LoopVariables = PrimaryExpr {',' PrimaryExpr} .
