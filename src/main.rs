use clap::clap_app;

fn main() -> anyhow::Result<()> {
    let mut app = clap_app!(music_info =>
        (@subcommand read =>
            (@arg JSON: +required -j --json "output json file")
            (@arg PICTURE: -p --picture "picture file path to write")
            (@arg AUDIOS: ... "source audio files")
        )
        (@subcommand write =>
            (@arg JSON: +required -j --json "read from json file")
            (@arg PICTURE: -p --picture "picture file path to write")
            (@arg AUDIOS: ... "target audio files")
        )
        (@subcommand search =>
            (@subcommand music_brainz =>
                (@arg QUERY: +required "query parameter to find info")
            )
        )
        (@subcommand fetch =>
            (@subcommand music_brainz =>
                (@arg Id: +required "MBID of target release")
                (@arg OUTPUT: -o --output "output file to save json")
                (@arg PICTURE: -p --picture "if present, save picture to PICTURE")
            )
        )
    );

    if std::env::args().len() == 1 {
        app.print_help()?;
        return Ok(())
    }
    
    let matches = app.get_matches();

    Ok(())
}
