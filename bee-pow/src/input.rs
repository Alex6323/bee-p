use common::constants::TRANSACTION_TRIT_LEN;
use common::Trit;

pub struct InputTrits(pub(self) [Trit; TRANSACTION_TRIT_LEN]);
