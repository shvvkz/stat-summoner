var searchIndex = new Map(JSON.parse('[\
["stat_summoner",{"t":"HHHCHCCCCHHHHCCCCCSFFFNNNNNNOONNNNNNNOONNNNNNNNOOOOOOOONNOOONNNNNNNNNNNNIFFNNNNNNNNNNOONNNNNNOOONNNNNNNNPPPPPPPPPPGPNNNNNNNNNNNNNNNNNNNCCCCCCHHCCHHHHHCCHHHHHHHHHHHHCCHHHHHHHHHHHHHHHHHH","n":["__loader","__runner","__shuttle_main","embed","main","models","module","riot_api","utils","create_embed","create_embed_error","create_embed_sucess","schedule_message_deletion","constants","data","error","modal","region","QUEUE_ID_MAP","Data","EmojiId","SummonerFollowedData","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","channel_id","dd_json","deserialize","deserialize","fmt","fmt","from","from","from","guild_id","id_emoji","into","into","into","into_request","into_request","into_request","into_resource","into_resource","last_match_id","mongo_client","name","name","puuid","region","riot_api_key","role","serialize","serialize","summoner_id","tag","time_end_follow","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","vzip","vzip","vzip","Error","FollowGamesModal","LolStatsModal","borrow","borrow","borrow_mut","borrow_mut","create","create","fmt","fmt","from","from","game_name","game_name","into","into","into_request","into_request","parse","parse","tag_line","tag_line","time_followed","try_from","try_from","try_into","try_into","type_id","type_id","vzip","vzip","BR","EUNE","EUW","JP","KR","LAN","LAS","NA","OCE","RU","Region","TR","borrow","borrow_mut","choices","create","extract","fmt","from","from_index","from_name","into","into_request","list","localized_name","name","pop_from","try_from","try_into","type_id","vzip","followgames","lolstats","loop_module","whoisfollowed","followgames","utils","followgames","check_and_add_in_db","lolstats","utils","lolstats","create_and_send_embed_lolstats","extract_champions_info","extract_match_info","extract_rank_info","loop_module","utils","check_and_update_db","delete_follower","get_followed_summoners","get_latest_match_id","is_follow_time_expired","process_followed_summoner","send_match_update_to_discord","update_follower_if_new_match","create_embed_loop","extract_participant_stats","format_gold_k","get_match_details","utils","whoisfollowed","create_embed_followed_summoner","format_duration","get_data_followed_summoner","whoisfollowed","get_champions","get_matchs_id","get_matchs_info","get_puuid","get_rank_info","get_summoner_id","open_dd_json","determine_solo_flex","get_emoji","get_game_mode","is_valid_game_mode","region_to_string","seconds_to_time","time_since_game_ended"],"q":[[0,"stat_summoner"],[9,"stat_summoner::embed"],[13,"stat_summoner::models"],[18,"stat_summoner::models::constants"],[19,"stat_summoner::models::data"],[72,"stat_summoner::models::error"],[73,"stat_summoner::models::modal"],[104,"stat_summoner::models::region"],[135,"stat_summoner::module"],[139,"stat_summoner::module::followgames"],[141,"stat_summoner::module::followgames::followgames"],[142,"stat_summoner::module::followgames::utils"],[143,"stat_summoner::module::lolstats"],[145,"stat_summoner::module::lolstats::lolstats"],[146,"stat_summoner::module::lolstats::utils"],[150,"stat_summoner::module::loop_module"],[152,"stat_summoner::module::loop_module::loop_module"],[160,"stat_summoner::module::loop_module::utils"],[164,"stat_summoner::module::whoisfollowed"],[166,"stat_summoner::module::whoisfollowed::utils"],[169,"stat_summoner::module::whoisfollowed::whoisfollowed"],[170,"stat_summoner::riot_api"],[177,"stat_summoner::utils"],[184,"shuttle_service"],[185,"alloc::vec"],[186,"shuttle_service::error"],[187,"core::result"],[188,"shuttle_serenity"],[189,"shuttle_common::secrets"],[190,"serde_json::value"],[191,"alloc::string"],[192,"mongodb::coll"],[193,"serenity::builder::create_embed"],[194,"core::error"],[195,"alloc::boxed"],[196,"poise::reply::builder"],[197,"poise::reply"],[198,"poise::structs::slash"],[199,"serde::de"],[200,"core::fmt"],[201,"tonic::request"],[202,"core::future::future"],[203,"core::pin"],[204,"serde::ser"],[205,"core::any"],[206,"core::option"],[207,"serenity::builder::create_interaction_response"],[208,"serenity::model::application::modal_interaction"],[209,"serenity::builder::create_command"],[210,"serenity::client::context"],[211,"serenity::model::application::command_interaction"],[212,"serenity::model::channel::message"],[213,"poise::structs::command"],[214,"std::collections::hash::map"],[215,"serde_json::map"],[216,"mongodb::client"],[217,"serenity::http::client"],[218,"alloc::sync"],[219,"mongodb::error"],[220,"reqwest::async_impl::client"],[221,"chrono"],[222,"serde::de::value"]],"i":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,21,24,13,21,24,13,24,21,24,13,24,13,21,24,13,24,13,21,24,13,21,24,13,24,13,24,21,24,13,24,24,21,13,24,13,24,24,24,21,24,13,21,24,13,21,24,13,21,24,13,0,0,0,9,35,9,35,9,35,9,35,9,35,9,35,9,35,9,35,9,35,9,35,35,9,35,9,35,9,35,9,35,42,42,42,42,42,42,42,42,42,42,0,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,42,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"f":"{b{{j{{f{{f{d}}}}h}}}}{{{f{{f{d}}}}}l}{nl}`{{}A`}````{{{Ad{Ab}}AfAfAh{f{Af}}{Al{Aj}}}{{j{An{Bb{B`}}}}}}{{{Ad{Bd}}}Bf}0{{Bh{Bl{Bj{Bb{B`}}}}}{{j{A`{Bb{B`}}}}}}`````````{{{Ad{c}}}{{Ad{e}}}{}{}}00{{{Ad{Bnc}}}{{Ad{Bne}}}{}{}}00``{c{{j{C`}}}Cb}{c{{j{Aj}}}Cb}{{{Ad{C`}}{Ad{BnCd}}}Cf}{{{Ad{Aj}}{Ad{BnCd}}}Cf}{cc{}}00``{ce{}{}}00{c{{Ch{e}}}{}{}}00{c{{Cl{{Bb{Cj}}}}}{}}0````````{{{Ad{C`}}c}jCn}{{{Ad{Aj}}c}jCn}```{c{{j{e}}}{}{}}00000{{{Ad{c}}}D`{}}00666```==<<{{{Db{Ab}}Ah}Dd}{{{Db{Df}}Ah}Dd}{{{Ad{Ab}}{Ad{BnCd}}}Cf}{{{Ad{Df}}{Ad{BnCd}}}Cf};;``::99{Dh{{j{Ab{Ad{Bd}}}}}}{Dh{{j{Df{Ad{Bd}}}}}}```777766<<````````````{{{Ad{c}}}{{Ad{e}}}{}{}}{{{Ad{Bnc}}}{{Ad{Bne}}}{}{}}{{}{{f{Dj}}}}{DlDl}{{{Ad{Dn}}{Ad{E`}}{Ad{Eb}}}{{Cl{{Bb{Cj}}}}}}{{{Ad{Ed}}{Ad{BnCd}}}Cf}{cc{}}{Ef{{Db{Ed}}}}{{{Ad{Bd}}}{{Db{Ed}}}}{ce{}{}}{c{{Ch{e}}}{}{}}8{{{Ad{Ed}}{Ad{Bd}}}{{Db{{Ad{Bd}}}}}}{{{Ad{Ed}}}{{Ad{Bd}}}}{{{Ad{Bd}}Ef{Ad{Dn}}{Ad{Eh}}}{{Cl{{Bb{Cj}}}}}}{c{{j{e}}}{}{}}0{{{Ad{c}}}D`{}}6``````{{}Ej}{{{Al{C`}}{Bl{Bj{Bb{B`}}}}DfAhAhAhAhAh}{{j{A`{Bb{B`}}}}}}``1{{{Ad{Ab}}Ah{Ad{{El{AhAf}}}}{Ad{{El{AhAf}}}}{f{{El{AhAf}}}}{f{Ah}}{Ad{{Bl{Bj{Bb{B`}}}}}}{Al{Aj}}}Bf}{{{f{{El{AhAf}}}}{Ad{{En{AhAf}}}}{Al{Aj}}}Ah}{{{f{Ah}}{Ad{{Bl{Bj{Bb{B`}}}}}}Ah}{{f{Af}}}}{{{Ad{{El{AhAf}}}}}Af}``{{{Ad{F`}}{Ad{Bd}}{Fd{Fb}}}{{j{A`{Bb{B`}}}}}}{{{Ad{{Al{C`}}}}{Ad{C`}}}{{j{A`Ff}}}}{{{Ad{{Al{C`}}}}}{{j{{f{C`}}Ff}}}}{{{Ad{Fh}}{Ad{Bd}}{Ad{Bd}}}{{j{Ah{Bb{B`}}}}}}{{{Ad{C`}}}Fj}{{{Ad{{Al{C`}}}}{Ad{C`}}{Ad{Bd}}{Fd{Fb}}{Al{Aj}}}{{j{A`{Bb{B`}}}}}}{{{Ad{C`}}{Ad{Bd}}{Ad{Bd}}{Ad{Bd}}{Fd{Fb}}{Al{Aj}}}{{j{A`{Bb{B`}}}}}}1{{{Ad{Af}}{Ad{Bd}}{Al{Aj}}}An}{{{Ad{Af}}}Af}{FlAh}{{{Ad{Af}}{Ad{Bd}}}{{Db{Af}}}}``{AfBf}{FnAh}{{{Al{C`}}Ah}{{j{Af{Bb{B`}}}}}}{{}Ej}{{{Ad{Fh}}{Ad{Bd}}{Ad{Bd}}{Ad{Bd}}}{{j{{f{{El{AhAf}}}}{Bb{B`}}}}}}{{{Ad{Fh}}{Ad{Bd}}{Ad{Bd}}G`}{{j{{f{Ah}}{Bb{B`}}}}}}{{{Ad{Fh}}{Ad{Bd}}{Ad{Bd}}}{{j{Af{Bb{B`}}}}}}{{{Ad{Fh}}{Ad{Bd}}{Ad{Bd}}{Ad{Bd}}}{{j{Ah{Bb{B`}}}}}}30{{}{{j{Af{Bb{B`}}}}}}{{{Ad{{f{{El{AhAf}}}}}}{Ad{{El{AhAf}}}}}{{Gb{{El{AhAf}}{El{AhAf}}}}}}{{{Al{Aj}}{Ad{Bd}}{Ad{Bd}}}{{j{AhGd}}}}{Gf{{Ad{Bd}}}}{GfFj}{{{Ad{Ed}}}Ah}{Fl{{Gb{AhAh}}}}{FlAh}","D":"Fl","p":[[5,"ResourceFactory",184],[1,"u8"],[5,"Vec",185],[6,"Error",186],[6,"Result",187],[8,"ShuttleSerenity",188],[5,"SecretStore",189],[1,"unit"],[5,"LolStatsModal",73],[1,"reference"],[6,"Value",190],[5,"String",191],[5,"EmojiId",19],[5,"Collection",192],[5,"CreateEmbed",193],[10,"Error",194],[5,"Box",195],[1,"str"],[5,"CreateReply",196],[5,"ReplyHandle",197],[5,"Data",19],[5,"ApplicationContext",198],[0,"mut"],[5,"SummonerFollowedData",19],[10,"Deserializer",199],[5,"Formatter",200],[8,"Result",200],[5,"Request",201],[10,"Future",202],[5,"Pin",203],[10,"Serializer",204],[5,"TypeId",205],[6,"Option",206],[6,"CreateInteractionResponse",207],[5,"FollowGamesModal",73],[5,"ModalInteractionData",208],[5,"CommandParameterChoice",198],[5,"CreateCommandOption",209],[5,"Context",210],[5,"CommandInteraction",211],[6,"ResolvedValue",211],[6,"Region",104],[1,"usize"],[5,"Message",212],[5,"Command",213],[5,"HashMap",214],[5,"Map",215],[5,"Client",216],[5,"Http",217],[5,"Arc",218],[5,"Error",219],[5,"Client",220],[1,"bool"],[1,"u64"],[8,"Duration",221],[1,"u32"],[1,"tuple"],[5,"Error",222],[1,"i64"]],"r":[],"b":[],"c":"OjAAAAAAAAA=","e":"OzAAAAEAAIIADgAAAAIABAAFAA4AFAAmAAEAKwAoAFYAAQBaACAAfAABAH8ADACNAAEAkQABAJcAAQClAAAAqgAAAA=="}]\
]'));
if (typeof exports !== 'undefined') exports.searchIndex = searchIndex;
else if (window.initSearch) window.initSearch(searchIndex);
