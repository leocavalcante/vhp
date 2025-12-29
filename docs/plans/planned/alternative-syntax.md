# Plan: Alternative Control Flow Syntax

## Overview

PHP supports an alternative syntax for control structures using colons instead of braces. This is commonly used in templates to improve readability when mixing PHP with HTML.

**PHP Example:**
```php
<?php if ($show): ?>
    <p>Content shown</p>
<?php elseif ($other): ?>
    <p>Other content</p>
<?php else: ?>
    <p>Default content</p>
<?php endif; ?>

<?php foreach ($items as $item): ?>
    <li><?= $item ?></li>
<?php endforeach; ?>

<?php while ($condition): ?>
    <p>Repeating...</p>
<?php endwhile; ?>

<?php for ($i = 0; $i < 10; $i++): ?>
    <span><?= $i ?></span>
<?php endfor; ?>

<?php switch ($value):
    case 1: ?>
        <p>One</p>
        <?php break; ?>
    <?php case 2: ?>
        <p>Two</p>
        <?php break; ?>
    <?php default: ?>
        <p>Other</p>
<?php endswitch; ?>
```

## Files to Modify

| File | Changes |
|------|---------|
| `src/token.rs` | Add `Endif`, `Endwhile`, `Endfor`, `Endforeach`, `Endswitch` tokens |
| `src/parser/stmt/mod.rs` | Parse alternative syntax |
| `tests/control_flow/alt_*.vhpt` | Test files |

Note: The interpreter doesn't need changes since the AST representation is the same.

## Implementation Steps

### Step 1: Add Tokens (`src/token.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens ...
    
    // Alternative syntax end keywords
    Endif,
    Endwhile,
    Endfor,
    Endforeach,
    Endswitch,
    
    // ... rest of tokens ...
}
```

### Step 2: Update Lexer (`src/lexer/mod.rs`)

```rust
fn tokenize_identifier(&mut self) -> TokenKind {
    match ident.to_lowercase().as_str() {
        // ... existing keywords ...
        "endif" => TokenKind::Endif,
        "endwhile" => TokenKind::Endwhile,
        "endfor" => TokenKind::Endfor,
        "endforeach" => TokenKind::Endforeach,
        "endswitch" => TokenKind::Endswitch,
        // ...
    }
}
```

### Step 3: Update If Parsing (`src/parser/stmt/mod.rs`)

```rust
fn parse_if(&mut self) -> Result<Stmt, String> {
    self.expect(&TokenKind::If)?;
    self.expect(&TokenKind::LeftParen)?;
    let condition = self.parse_expression()?;
    self.expect(&TokenKind::RightParen)?;
    
    // Check for alternative syntax: if (...):
    let is_alternative = if self.check(&TokenKind::Colon) {
        self.advance();
        true
    } else {
        false
    };
    
    if is_alternative {
        self.parse_if_alternative(condition)
    } else {
        self.parse_if_brace(condition)
    }
}

fn parse_if_alternative(&mut self, condition: Expr) -> Result<Stmt, String> {
    // Parse then branch until elseif/else/endif
    let mut then_branch = vec![];
    let mut else_branch = None;
    let mut elseif_branches = vec![];
    
    loop {
        // Check for elseif, else, or endif
        if self.check(&TokenKind::Elseif) {
            self.advance();
            self.expect(&TokenKind::LeftParen)?;
            let elseif_cond = self.parse_expression()?;
            self.expect(&TokenKind::RightParen)?;
            self.expect(&TokenKind::Colon)?;
            
            let elseif_body = self.parse_statements_until_control_end()?;
            elseif_branches.push((elseif_cond, elseif_body));
            continue;
        }
        
        if self.check(&TokenKind::Else) {
            self.advance();
            // Could be `else:` or `else if` (two tokens)
            if self.check(&TokenKind::If) {
                // Treat as elseif
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let elseif_cond = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                self.expect(&TokenKind::Colon)?;
                
                let elseif_body = self.parse_statements_until_control_end()?;
                elseif_branches.push((elseif_cond, elseif_body));
                continue;
            }
            
            self.expect(&TokenKind::Colon)?;
            else_branch = Some(self.parse_statements_until_control_end()?);
            continue;
        }
        
        if self.check(&TokenKind::Endif) {
            self.advance();
            self.expect(&TokenKind::Semicolon)?;
            break;
        }
        
        // Parse statement
        if let Some(stmt) = self.parse_statement()? {
            then_branch.push(stmt);
        } else {
            break;
        }
    }
    
    // Build nested if structure from elseif branches
    let final_else = else_branch;
    let mut result_else = final_else;
    
    // Process elseif branches in reverse to build nested structure
    for (cond, body) in elseif_branches.into_iter().rev() {
        result_else = Some(vec![Stmt::If {
            condition: cond,
            then_branch: body,
            else_branch: result_else,
        }]);
    }
    
    Ok(Stmt::If {
        condition,
        then_branch,
        else_branch: result_else,
    })
}

/// Parse statements until we see an alternative syntax end keyword
fn parse_statements_until_control_end(&mut self) -> Result<Vec<Stmt>, String> {
    let mut stmts = vec![];
    
    while !self.is_at_control_end() && !self.is_at_end() {
        if let Some(stmt) = self.parse_statement()? {
            stmts.push(stmt);
        }
    }
    
    Ok(stmts)
}

fn is_at_control_end(&self) -> bool {
    matches!(
        &self.current_token().kind,
        TokenKind::Endif | TokenKind::Endwhile | TokenKind::Endfor |
        TokenKind::Endforeach | TokenKind::Endswitch |
        TokenKind::Elseif | TokenKind::Else |
        TokenKind::Case | TokenKind::Default
    )
}
```

### Step 4: Update While Parsing

```rust
fn parse_while(&mut self) -> Result<Stmt, String> {
    self.expect(&TokenKind::While)?;
    self.expect(&TokenKind::LeftParen)?;
    let condition = self.parse_expression()?;
    self.expect(&TokenKind::RightParen)?;
    
    // Check for alternative syntax: while (...):
    if self.check(&TokenKind::Colon) {
        self.advance();
        let body = self.parse_statements_until(&TokenKind::Endwhile)?;
        self.expect(&TokenKind::Endwhile)?;
        self.expect(&TokenKind::Semicolon)?;
        return Ok(Stmt::While { condition, body });
    }
    
    // Brace syntax
    self.expect(&TokenKind::LeftBrace)?;
    let body = self.parse_block()?;
    self.expect(&TokenKind::RightBrace)?;
    
    Ok(Stmt::While { condition, body })
}
```

### Step 5: Update For Parsing

```rust
fn parse_for(&mut self) -> Result<Stmt, String> {
    self.expect(&TokenKind::For)?;
    self.expect(&TokenKind::LeftParen)?;
    
    // Parse init, condition, update
    let init = self.parse_for_init()?;
    self.expect(&TokenKind::Semicolon)?;
    let condition = self.parse_for_condition()?;
    self.expect(&TokenKind::Semicolon)?;
    let update = self.parse_for_update()?;
    
    self.expect(&TokenKind::RightParen)?;
    
    // Check for alternative syntax: for (...):
    if self.check(&TokenKind::Colon) {
        self.advance();
        let body = self.parse_statements_until(&TokenKind::Endfor)?;
        self.expect(&TokenKind::Endfor)?;
        self.expect(&TokenKind::Semicolon)?;
        return Ok(Stmt::For { init, condition, update, body });
    }
    
    // Brace syntax
    self.expect(&TokenKind::LeftBrace)?;
    let body = self.parse_block()?;
    self.expect(&TokenKind::RightBrace)?;
    
    Ok(Stmt::For { init, condition, update, body })
}
```

### Step 6: Update Foreach Parsing

```rust
fn parse_foreach(&mut self) -> Result<Stmt, String> {
    self.expect(&TokenKind::Foreach)?;
    self.expect(&TokenKind::LeftParen)?;
    
    let iterable = self.parse_expression()?;
    self.expect(&TokenKind::As)?;
    
    let (key_var, value_var) = self.parse_foreach_vars()?;
    
    self.expect(&TokenKind::RightParen)?;
    
    // Check for alternative syntax: foreach (...):
    if self.check(&TokenKind::Colon) {
        self.advance();
        let body = self.parse_statements_until(&TokenKind::Endforeach)?;
        self.expect(&TokenKind::Endforeach)?;
        self.expect(&TokenKind::Semicolon)?;
        return Ok(Stmt::Foreach { iterable, key_var, value_var, body });
    }
    
    // Brace syntax
    self.expect(&TokenKind::LeftBrace)?;
    let body = self.parse_block()?;
    self.expect(&TokenKind::RightBrace)?;
    
    Ok(Stmt::Foreach { iterable, key_var, value_var, body })
}
```

### Step 7: Update Switch Parsing

```rust
fn parse_switch(&mut self) -> Result<Stmt, String> {
    self.expect(&TokenKind::Switch)?;
    self.expect(&TokenKind::LeftParen)?;
    let expr = self.parse_expression()?;
    self.expect(&TokenKind::RightParen)?;
    
    // Check for alternative syntax: switch (...):
    let is_alternative = if self.check(&TokenKind::Colon) {
        self.advance();
        true
    } else {
        self.expect(&TokenKind::LeftBrace)?;
        false
    };
    
    let mut cases = vec![];
    let mut default = None;
    
    loop {
        if is_alternative && self.check(&TokenKind::Endswitch) {
            self.advance();
            self.expect(&TokenKind::Semicolon)?;
            break;
        }
        
        if !is_alternative && self.check(&TokenKind::RightBrace) {
            self.advance();
            break;
        }
        
        if self.check(&TokenKind::Case) {
            self.advance();
            let value = self.parse_expression()?;
            // Accept both : and ; after case
            if self.check(&TokenKind::Colon) || self.check(&TokenKind::Semicolon) {
                self.advance();
            } else {
                return Err("Expected ':' or ';' after case value".to_string());
            }
            
            let body = self.parse_case_body(is_alternative)?;
            cases.push((value, body));
        } else if self.check(&TokenKind::Default) {
            self.advance();
            if self.check(&TokenKind::Colon) || self.check(&TokenKind::Semicolon) {
                self.advance();
            } else {
                return Err("Expected ':' or ';' after default".to_string());
            }
            
            default = Some(self.parse_case_body(is_alternative)?);
        } else {
            break;
        }
    }
    
    Ok(Stmt::Switch { expr, cases, default })
}

fn parse_case_body(&mut self, is_alternative: bool) -> Result<Vec<Stmt>, String> {
    let mut stmts = vec![];
    
    loop {
        // Stop at next case, default, or end
        if self.check(&TokenKind::Case) || self.check(&TokenKind::Default) {
            break;
        }
        
        if is_alternative && self.check(&TokenKind::Endswitch) {
            break;
        }
        
        if !is_alternative && self.check(&TokenKind::RightBrace) {
            break;
        }
        
        if let Some(stmt) = self.parse_statement()? {
            stmts.push(stmt);
        } else {
            break;
        }
    }
    
    Ok(stmts)
}
```

### Step 8: Add Tests

**tests/control_flow/alt_if.vhpt**
```
--TEST--
Alternative if syntax
--FILE--
<?php
$x = 1;
if ($x == 1):
    echo "one";
endif;
--EXPECT--
one
```

**tests/control_flow/alt_if_else.vhpt**
```
--TEST--
Alternative if-else syntax
--FILE--
<?php
$x = 2;
if ($x == 1):
    echo "one";
else:
    echo "not one";
endif;
--EXPECT--
not one
```

**tests/control_flow/alt_if_elseif.vhpt**
```
--TEST--
Alternative if-elseif-else syntax
--FILE--
<?php
$x = 2;
if ($x == 1):
    echo "one";
elseif ($x == 2):
    echo "two";
else:
    echo "other";
endif;
--EXPECT--
two
```

**tests/control_flow/alt_while.vhpt**
```
--TEST--
Alternative while syntax
--FILE--
<?php
$i = 0;
while ($i < 3):
    echo $i;
    $i++;
endwhile;
--EXPECT--
012
```

**tests/control_flow/alt_for.vhpt**
```
--TEST--
Alternative for syntax
--FILE--
<?php
for ($i = 1; $i <= 3; $i++):
    echo $i;
endfor;
--EXPECT--
123
```

**tests/control_flow/alt_foreach.vhpt**
```
--TEST--
Alternative foreach syntax
--FILE--
<?php
$arr = ["a", "b", "c"];
foreach ($arr as $v):
    echo $v;
endforeach;
--EXPECT--
abc
```

**tests/control_flow/alt_foreach_key_value.vhpt**
```
--TEST--
Alternative foreach with key-value
--FILE--
<?php
$arr = ["x" => 1, "y" => 2];
foreach ($arr as $k => $v):
    echo "$k=$v ";
endforeach;
--EXPECT--
x=1 y=2 
```

**tests/control_flow/alt_switch.vhpt**
```
--TEST--
Alternative switch syntax
--FILE--
<?php
$x = 2;
switch ($x):
    case 1:
        echo "one";
        break;
    case 2:
        echo "two";
        break;
    default:
        echo "other";
endswitch;
--EXPECT--
two
```

**tests/control_flow/alt_nested.vhpt**
```
--TEST--
Nested alternative syntax
--FILE--
<?php
$items = [1, 2];
foreach ($items as $item):
    if ($item == 1):
        echo "first";
    else:
        echo "other";
    endif;
    echo "\n";
endforeach;
--EXPECT--
first
other

```

**tests/control_flow/alt_with_html.vhpt**
```
--TEST--
Alternative syntax with HTML passthrough
--FILE--
<?php $show = true; ?>
<?php if ($show): ?>
<p>Shown</p>
<?php endif; ?>
--EXPECT--
<p>Shown</p>
```

**tests/control_flow/alt_else_if_two_tokens.vhpt**
```
--TEST--
Alternative syntax with else if (two tokens)
--FILE--
<?php
$x = 2;
if ($x == 1):
    echo "one";
else if ($x == 2):
    echo "two";
endif;
--EXPECT--
two
```

## Key Rules

1. **Colon starts**: `if (...):` instead of `if (...) {`
2. **End keywords**: `endif;`, `endwhile;`, `endfor;`, `endforeach;`, `endswitch;`
3. **Semicolon required**: End keywords must be followed by `;`
4. **Mixing forbidden**: Cannot mix brace and alternative syntax in same block
5. **Case separators**: Both `:` and `;` allowed after case values

## PHP Compatibility Notes

| Feature | PHP Version |
|---------|-------------|
| Alternative syntax for all control structures | 4.0 |
| Semicolon after case value | 7.0+ |

## Implementation Order

1. Add end tokens
2. If/elseif/else/endif
3. While/endwhile
4. For/endfor
5. Foreach/endforeach
6. Switch/endswitch
7. Nested alternative syntax
8. HTML passthrough integration

## Error Messages

- `syntax error, unexpected end of file, expecting endif`
- `Cannot mix braced and alternative syntax`
- `Expected ':' or ';' after case value`
