use crate::player::vlc::start_vlc::*;
use crate::player::vlc::fetch_vlc_data::*;
use crate::api::me::update_media_progress::*;
use crate::api::library_items::play_lib_item_or_pod::*;

// handle l when is_podact is true for continue listening `AppView::Home`
pub async fn handle_l_pod_home(
    token: Option<&String>,
    ids_library_items: &Vec<String>,
    selected: Option<usize>,
    port: String,
    id_pod: Vec<String>,
) {
    if let Some(index) = selected {
        // id is id of the podcast  and id_pod_ep is the id id of the episode podcast
        if let Some(id) = ids_library_items.get(index) {
            if let Some(id_pod_ep) = id_pod.get(index) {
                if let Some(token) = token {
                    if let Ok(info_item) = post_start_playback_session_pod(Some(&token), &id, id_pod_ep).await {
                    // clone otherwise, these variable will  be consumed and not available anymore
                // for use outside start_vlc spawn
                let token_clone = token.clone();
                let port_clone = port.clone();
                let info_item_clone = info_item.clone() ;
                // Start VLC is launched in a spawn to allow fetch_vlc_data to start at the same time
                tokio::spawn(async move {
                        start_vlc(&info_item_clone[0], &port_clone, &info_item_clone[1], Some(&token_clone)).await;
                    });

                    // Important, sleep time to 1s otherwise connection to vlc player will not have time to connect
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                    loop {
                        match fetch_vlc_data(port.clone()).await {
                            Ok(Some(data_fetched_from_vlc)) => {
                                //println!("Fetched data: {}", data_fetched.to_string());

                                // Important, sleep time to 1s otherwise connection to vlc player will not have time to connect
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                match fetch_vlc_is_playing(port.clone()).await {
                                    Ok(true) => {
                                        let _ = update_media_progress_pod(id, Some(&token), Some(data_fetched_from_vlc), &info_item[2], &id_pod_ep).await;
                                        //println!("{:?}", data_fetched_from_vlc);
                                    },
                                    Ok(false) => {
                                        let is_finised = true;
                                        let _ = update_media_progress2_pod(id, Some(&token), Some(data_fetched_from_vlc), &info_item[2], is_finised, &id_pod_ep).await;
                                        break; 
                                    },
                                    Err(_e) => {
                                        //eprintln!("Error fetching play status: {}", e);
                                        break; 
                                    }
                                }

                            }
                            Ok(None) => {
                                break; // Exit if no data available
                            }
                            Err(_e) => {
                                break; // Exit on error
                            }
                        }
                    }
                } else {
                    eprintln!("Failed to start playback session");
                }
            }
        }
    }
}}

