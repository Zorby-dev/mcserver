use crate::macros::packets;

packets!{
    handshaking {
        serverbound {
            0x00 => Handshake { protocol_version: VarI32, server_address: String, server_port: u16, next_state: VarI32 }
        }
    }
    status {
        serverbound {
            0x00 => StatusRequest { },
            0x01 => PingRequest { payload: i64 }
        }
        clientbound {
            0x00 => StatusResponse { response: String },
            0x01 => PingResponse { payload: i64 }
        }
    }
    login {
        serverbound {
            0x00 => LoginStart { name: String, uuid: Option::<UUID> }
        }
        clientbound {
            // FIXME: implement properties properly
            //        https://wiki.vg/Protocol#Login_Success
            0x02 => LoginSuccess { uuid: UUID, username: String, number_of_properties: VarI32 }
        }
    }
    play {
        clientbound {
            0x1a => Disconnect { reason: String },
            // FIXME: implement nbt AND death position properly
            0x28 => Login { entity_id: i32, is_hardcore: bool, gamemode: u8, previous_gamemode: i8, dimensions: Vec::<Identifier>, registry_codec: Nbt, dimension_type: Identifier, dimension_name: Identifier, hashed_seed: i64, max_players: VarI32, render_distance: VarI32, simulation_distance: VarI32, reduced_debug_info: bool, enable_respawn_screen: bool, is_debug: bool, is_flat: bool, has_death_location: bool },
            0x4d => SetHeldItem { slot: i8 },
            0x50 => SetDefaultSpawnPosition { location: Pos, angle: f32 },
            // FIXME: Implement properly
            0x6d => UpdateRecipes { recipes_count: VarI32 },
            // FIXME: Implement properly
            0x6e => UpdateTags { tags_count: VarI32 }
        }
    }
}