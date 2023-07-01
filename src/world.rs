use crate::{interface::Interface, packets::play::clientbound::{self, LoginData, SetDefaultSpawnPositionData}, types::{Nbt, Identifier, Pos}};

pub struct World {
    players: Vec<Interface>
}

impl World {
    pub fn new() -> Self {
        Self {
            players: vec![]
        }
    }

    pub async fn connect_client(&mut self, mut interface: Interface) {
        interface.send(clientbound::Packet::Login(LoginData {
            entity_id: 0,
            is_hardcore: false,
            gamemode: 0,
            previous_gamemode: 0,
            dimensions: vec![Identifier::from("minecraft:overworld"), Identifier::from("minecraft:the_nether"), Identifier::from("minecraft:the_end")],
            registry_codec: Nbt(Vec::from(*include_bytes!("./RegistryCodec.nbt"))),
            dimension_type: "minecraft:overworld".into(),
            dimension_name: "minecraft:overworld".into(),
            hashed_seed: 0,
            max_players: 1.into(),
            render_distance: 5.into(),
            simulation_distance: 5.into(),
            reduced_debug_info: false,
            enable_respawn_screen: true,
            is_debug: false,
            is_flat: true,
            has_death_location: false
        })).await;

        /*interface.send(clientbound::Packet::SetHeldItem(SetHeldItemData {
            slot: 0
        })).await;

        interface.send(clientbound::Packet::UpdateRecipes(UpdateRecipesData {
            recipes_count: 0.into()
        })).await;

        interface.send(clientbound::Packet::UpdateTags(UpdateTagsData {
            tags_count: 0.into()
        })).await;*/

        interface.send(clientbound::Packet::SetDefaultSpawnPosition(SetDefaultSpawnPositionData {
            location: Pos { x: 0, y: 0, z: 0 },
            angle: 0.0
        })).await;

        self.players.push(interface);

        /*let packet = clientbound::Packet::Disconnect(DisconnectData {
            reason: String::from("[{\"text\":\"[Intro]\n\",\"color\":\"gray\",\"underlined\":true,\"obfuscated\":true},{\"text\":\"Desert you\nOoh-ooh-ooh-ooh\nHurt you\n\n\",\"color\":\"white\",\"underlined\":false},{\"text\":\"[Verse 1]\n\",\"color\":\"gray\",\"underlined\":true},{\"text\":\"We're no strangers to love\nYou know the rules and so do I\nA full commitment's what I'm thinking of\nYou wouldn't get this from any other guy\n\n\",\"color\":\"white\",\"underlined\":false},{\"text\":\"[Pre-Chorus]\n\",\"color\":\"gray\",\"underlined\":true},{\"text\":\"I just wanna tell you how I'm feeling\nGotta make you understand\n\n\",\"color\":\"white\",\"underlined\":false},{\"text\":\"[Chorus]\n\",\"color\":\"gray\",\"underlined\":true},{\"text\":\"Never gonna give you up\nNever gonna let you down\nNever gonna run around and desert you\nNever gonna make you cry\nNever gonna say goodbye\nNever gonna tell a lie and hurt you\n\n\",\"color\":\"white\",\"underlined\":false},{\"text\":\"[Verse 2]\n\",\"color\":\"gray\",\"underlined\":true},{\"text\":\"We've known each other for so long\nYour heart's been aching, but you're too shy to say it\nInside, we both know what's been going on\nWe know the game, and we're gonna play it\n\n\",\"color\":\"white\",\"underlined\":false},{\"text\":\"[Pre-Chorus]\n\",\"color\":\"gray\",\"underlined\":true},{\"text\":\"And if you ask me how I'm feeling\nDon't tell me you're too blind to see\n\n\",\"color\":\"white\",\"underlined\":false},{\"text\":\"[Chorus]\n\",\"color\":\"gray\",\"underlined\":true},{\"text\":\"Never gonna give you up\nNever gonna let you down\nNever gonna run around and desert you\nNever gonna make you cry\nNever gonna say goodbye\nNever gonna tell a lie and hurt you\nNever gonna give you up\nNever gonna let you down\nNever gonna run around and desert you\nNever gonna make you cry\nNever gonna say goodbye\nNever gonna tell a lie and hurt you\n\n\",\"color\":\"red\",\"underlined\":false,\"bold\":true},{\"text\":\"[Post-Chorus]\n\",\"color\":\"gray\",\"underlined\":true,\"bold\":false},{\"text\":\"Ooh (Give you up)\nOoh-ooh (Give you up)\nOoh-ooh\nNever gonna give, never gonna give (Give you up)\nOoh-ooh\nNever gonna give, never gonna give (Give you up)\n\n\",\"color\":\"white\",\"underlined\":false},{\"text\":\"[Bridge]\n\",\"color\":\"gray\",\"underlined\":true},{\"text\":\"We've known each other for so long\nYour heart's been aching, but you're too shy to say it\nInside, we both know what's been going on\nWe know the game, and we're gonna play it\n\n\",\"color\":\"white\",\"underlined\":false},{\"text\":\"[Pre-Chorus]\n\",\"color\":\"gray\",\"underlined\":true},{\"text\":\"I just wanna tell you how I'm feeling\nGotta make you understand\n\n\",\"color\":\"white\",\"underlined\":false},{\"text\":\"[Chorus]\n\",\"color\":\"gray\",\"underlined\":true},{\"text\":\"Never gonna give you up\nNever gonna let you down\nNever gonna run around and desert you\nNever gonna make you cry\nNever gonna say goodbye\nNever gonna tell a lie and hurt you\nNever gonna give you up\nNever gonna let you down\nNever gonna run around and desert you\nNever gonna make you cry\nNever gonna say goodbye\nNever gonna tell a lie and hurt you\nNever gonna give you up\nNever gonna let you down\nNever gonna run around and desert you\nNever gonna make you cry\nNever gonna say goodbye\nNever gonna tell a lie and hurt you\",\"color\":\"white\",\"underlined\":false}]")
        });

        interface.send(packet).await;
        interface.disconnect().await;*/
    }
}