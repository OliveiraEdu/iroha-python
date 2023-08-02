
from ...rust import make_enum, make_struct, make_tuple, get_class, SelfResolvingTypeVar, Dict
import typing
            
BlockHeaderValue = make_struct("BlockHeaderValue", [("timestamp", int), ("height", int), ("previous_block_hash", "iroha_crypto.hash.Hash"), ("transactions_hash", "iroha_crypto.hash.HashOf"), ("rejected_transactions_hash", "iroha_crypto.hash.HashOf"), ("invalidated_blocks_hashes", list), ("current_block_hash", "iroha_crypto.hash.Hash")])

BlockValue = make_struct("BlockValue", [("header", "iroha_data_model.block_value.BlockHeaderValue"), ("transactions", list), ("rejected_transactions", list), ("event_recommendations", list)])

SelfResolvingTypeVar.resolve_all()