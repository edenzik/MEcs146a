use std::mem::replace;

fn main() {
    let input = "cat hello|grep moo";
    let x = GashCommandLine::new(input.as_slice());
    match x {
        GashCommandLine::Foreground(v) => {
            for a in v{
                match a{
                    GashCommand::Normal(l) => println!("{:?}", l.operator),
                    _ => println!("")
                }
            }
        }
        _ => {}
    }
}


enum GashCommandLine<'a> {
    Foreground(Vec<GashCommand<'a>>),
    Background(Vec<GashCommand<'a>>),
    Empty
}

impl<'a> GashCommandLine<'a> {
    fn new(line : & 'a str) -> GashCommandLine<'a> {
        let mut commands = Vec::new();
        for command_str in line.split('|'){
            commands.push(GashCommand::new(command_str));
        }
        match line.chars().last().unwrap(){
            '&' => GashCommandLine::Background(commands),
            _ if !commands.is_empty() => GashCommandLine::Foreground(commands),
            _ => GashCommandLine::Empty            
        }
    }
}

impl<'a> GashCommand<'a> {
    fn new(command : & 'a str) -> GashCommand<'a> {
        let mut tokens = command.words();
        let operator = tokens.next().unwrap();
        match operator {
            "cd" => return GashCommand::ChangeDirectory(Box::new(command.words().next().unwrap())),

            "history" =>        return GashCommand::History,

            _   if command.contains(">") => {
                let mut command = command.split_str(">");
                let mut tokens = command.next().unwrap().words();
                let operator = tokens.next().unwrap();
                GashCommand::OutputRedirect(GashOperation{operator:Box::new(operator), operands:Box::new(tokens.collect())}, Box::new(command.next().unwrap()));
            },

            _   =>  return GashCommand::Normal(GashOperation{operator:Box::new(operator),operands:Box::new(tokens.collect())}),
        }
        GashCommand::BadCommand
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
    InputRedirect(GashOperation<'a>, Box<& 'a str>),
    OutputRedirect(GashOperation<'a>, Box<& 'a str>),
    BadCommand,
}

