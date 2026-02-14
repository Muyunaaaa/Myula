pub mod ir;
pub mod lexer;
pub mod parser;

#[cfg(test)]
mod tests {
    use crate::frontend::ir::IRGenerator;
    use crate::frontend::lexer::Lexer;
    use crate::frontend::parser::Parser;

    #[test]
    fn it_works() {
        let mut lexer = Lexer::new(
            "
        local function hello_world()
            print(\"Hello, World!\")
        end
        local function fake_closure()
            return function(x, y)
                return x + y
            end
        end
        if true then
            local test = 123
        else
            local test = 456
        end
        y = 114514.1919810
        z = 123
        local x = 10 + 20 * (30 - 5)
        print(w()[1 + 1]()[42][\"hello\"])
        local out = 0
        if x >= 100 then
            out = 0
            print(out)
        else
            out = 1
            print(456)
        end
        while x < 200 do
            x = x + 1
        end
        repeat
            x = x + 2
        until x >= 300
        local tbl = {a = 1, b = 2, c = 3, 4, 5, 6, [ 1 + 1 ] = 42}
        tbl.hello = \"world\"
        tbl[2] = 3
        print(tbl[\"hello\"])
        print(tbl.a)
        print(tbl[\"b\"])
        print(tbl[1])
        ",
        );
        let mut parser = Parser::new(&mut lexer);

        let ast = parser.parse();
        println!("{:#?}", ast);
        println!("Lexer Errors: {:#?}", parser.get_lexer().get_err());
        println!("Parser Errors: {:#?}", parser.get_err());

        let mut ir_gen = IRGenerator::new();
        ir_gen.generate(&ast);
        println!("{}", ir_gen.get_module().to_string());
        println!("IR Generation Errors: {:#?}", ir_gen.get_err());
    }
}
