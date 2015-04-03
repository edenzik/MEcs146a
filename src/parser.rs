fn main() {

}


enum GashCommandLine<'a> {
    Foreground(Vec<GashCommand<'a>>),
    Background(Vec<GashCommand<'a>>),
    Empty
}

impl<'a> GashCommandLine<'a> {
    fn new(line : & 'a str) -> GashCommandLine<'a> {
        let mut d = String::new();
        let mut buffer = String::new();
        let mut chars_iter = line.chars();
        let mut commands = Vec::new();
        for c in chars_iter{
           // d = buffer.clone();
            match c {
                '|' => {
                    let s = &mut buffer;
                    commands.push(GashCommand::new(Box::new(s)));
                    let buffer =  &mut String::new();
                },
                _   =>  buffer.push(c),
            }
        }
        GashCommandLine::Empty


    }
}



struct GashOperation<'a>{
    operator : Box<& 'a str>,
    operands: Box<Vec<& 'a str>>
}



enum GashCommand<'a> {
    Normal(GashOperation<'a>),
    History,
    ChangeDirectory(Box<& 'a str>),
    InputRedirect(GashOperation<'a>, String),
    OutputRedirect(GashOperation<'a>, Box<& 'a str>),
    BadCommand,
}

impl<'a> GashCommand<'a> {
    fn new(command : Box<& 'a str>) -> GashCommand<'a> {
        let mut tokens = &mut command.words();
        let operator = tokens.next().unwrap();
        match operator {
            "cd" => {           return GashCommand::ChangeDirectory(Box::new(tokens.next().unwrap()));}

            "history" =>        return GashCommand::History,

            _ if command.contains(">") => {
                let operands = Box::new(tokens.take_while(|&c| c!=">").collect());
                let operation = GashOperation{operator:Box::new(operator), operands:operands}; 
                return GashCommand::OutputRedirect(operation,Box::new(tokens.next().unwrap()));}

            _   =>  {           return GashCommand::Normal(GashOperation{operator:Box::new(operator),operands:Box::new(tokens.collect())})}
        }
        GashCommand::History
    }
}






