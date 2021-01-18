// use mocknet::algo::in_memory_graph::{*};

macro_rules! define_config_directive {
    // Start rule.
    // Note: `$(,)*` is a trick to eat any number of trailing commas.
    ( $( {$($cmd:tt)*} ),* $(,)*) => {
        // This starts the parse, giving the initial state of the output
        // (i.e. empty).  Note that the commands come after the semicolon.
        define_config_directive! { @parse {}, (args){}; $({$($cmd)*},)* }
    };

    // Termination rule: no more input.
    (
        @parse
        // $eout will be the body of the enum.
        {$($eout:tt)*},
        // $pout will be the body of the `parse_line` match.
        // We pass `args` explicitly to make sure all stages are using the
        // *same* `args` (due to identifier hygiene).
        ($args:ident){$($pout:tt)*};
        // See, nothing here?
    ) => {
        #[derive(PartialEq, Eq, Debug)]
        enum ConfigDirective {
            $($eout)*
        }

        fn parse_line(command: &str, $args: &[&str]) -> ConfigDirective {
            match command {
                $($pout)*
                _ => panic!("unknown command: {:?}", command)
            }
        }
    };

    // Rule for command with no arguments.
    (
        @parse {$($eout:tt)*}, ($pargs:ident){$($pout:tt)*};
        {
            command: $sname:expr,
            rust_name: $rname:ident,
            args: [],
            optional_args: [] $(,)*
        },
        $($tail:tt)*
    ) => {
        define_config_directive! {
            @parse
            {
                $($eout)*
                $rname,
            },
            ($pargs){
                $($pout)*
                $sname => ConfigDirective::$rname,
            };
            $($tail)*
        }
    };

    // Rule for other commands.
    (
        @parse {$($eout:tt)*}, ($pargs:ident){$($pout:tt)*};
        {
            command: $sname:expr,
            rust_name: $rname:ident,
            args: [$($args:ident),* $(,)*],
            optional_args: [$($oargs:ident),* $(,)*] $(,)*
        },
        $($tail:tt)*
    ) => {
        define_config_directive! {
            @parse
            {
                $($eout)*
                $rname { $( $args: String, )* $( $oargs: Option<String>, )* },
            },
            ($pargs){
                $($pout)*
                $sname => {
                    // This trickery is because macros can't count with
                    // regular integers.  We'll just use a mutable index
                    // instead.
                    let mut i = 0;
                    $(let $args = $pargs[i].into(); i += 1;)*
                    $(let $oargs = $pargs.get(i).map(|&s| s.into()); i += 1;)*
                    let _ = i; // avoid unused assignment warnings.

                    ConfigDirective::$rname {
                        $($args: $args,)*
                        $($oargs: $oargs,)*
                    }
                },
            };
            $($tail)*
        }
    };
}

define_config_directive! {
    {command: "command1", rust_name: CommandOne, args: [arg1], optional_args: []},
    {command: "other_command", rust_name: OtherCommand, args: [arg1], optional_args: [optional_arg1]},
}

fn main() {
    // // print_an_address().unwrap();
    // println!("mf");
    // // let vertexes: Vec<_> = vec!(1,2,3,4,5).into_iter().map(|e| {
    // //     (e, 0)
    // // }).collect();
    // // let edges: Vec<_> = vec!((1,2), (1,3), (1,4), (1,5), (2,3)).into_iter().map(|e| {
    // //     (e, 0)
    // // }).collect();

    // // let vertexes: Vec<_> = vec!(1,2,3,3,4,5).into_iter().map(|e| {
    // //     (e, 0)
    // // }).collect();
    // // let edges: Vec<_> = vec!((1,2), (1,3), (1,4), (1,5), (2,3)).into_iter().map(|e| {
    // //     (e, 0)
    // // }).collect();

    // let vertexes: Vec<_> = vec!(1,2,3,4,5).into_iter().map(|e| {
    //     (e, 0)
    // }).collect();
    // let edges: Vec<_> = vec!((1,2), (1,3), (1,4), (1,4), (1,5), (2,3), (7,8)).into_iter().map(|e| {
    //     (e, 0)
    // }).collect();

    // let graph: InMemoryGraph<u64, u64, u64> = InMemoryGraph::from_vecs(vertexes, edges).unwrap();
    // graph.dump();

    println!("{:?}", parse_line("command1", &["foo"]));
    println!("{:?}", parse_line("other_command", &["foo"]));
    println!("{:?}", parse_line("other_command", &["foo", "bar"]));
}
