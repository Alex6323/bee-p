use common::constants::*;
use common::Errors;
use common::Result;
use common::Tryte;

use ternary::IsTryte;

use crate::constants::*;

use std::fmt;
use std::hash::Hash as StdHash;
use std::hash::Hasher as StdHasher;

macro_rules! implement_debug {
    ($($t:ty),+) => {
    $(
        impl fmt::Debug for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0.iter().collect::<String>())
            }
        }
    )+
    }
}

macro_rules! implement_display {
    ($($t:ty),+) => {
    $(
        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0.iter().collect::<String>())
            }
        }
    )+
    }
}

macro_rules! implement_hash {
    ($($t:ty),+) => {
    $(
        impl StdHash for $t {
            fn hash<H : StdHasher>(&self, state: &mut H) {
                    self.0.hash(state);
            }
        }
    )+
    }
}

macro_rules! implement_eq {
    ($($t:ty),+) => {
    $(
        impl PartialEq for $t {
            fn eq(&self,other: &$t) -> bool {
                    self.0.iter().zip(other.0.iter()).all(|(a,b)| a == b)
            }
        }

        impl Eq for $t {}

    )+
    }
}

macro_rules! implement_clone {
    ($($t:ty),+) => {
    $(
        impl Clone for $t {
            fn clone(&self) -> Self {
                    let mut cloned : $t = <$t>::default();
                    cloned.0 = self.0;
                    cloned
            }
        }

    )+
    }
}

pub struct Payload(pub [Tryte; PAYLOAD.tryte_offset.length]);

impl Payload {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(payload: &str) -> Self {
        assert!(payload.len() <= PAYLOAD.tryte_offset.length);
        assert!(payload.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; PAYLOAD.tryte_offset.length];

        for (i, c) in payload.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self([TRYTE_ZERO; PAYLOAD.tryte_offset.length])
    }
}

pub struct Address(pub [Tryte; ADDRESS.tryte_offset.length]);

impl Address {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(address: &str) -> Self {
        assert!(address.len() <= ADDRESS.tryte_offset.length);
        assert!(address.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; ADDRESS.tryte_offset.length];

        for (i, c) in address.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}

impl Default for Address {
    fn default() -> Self {
        Self([TRYTE_ZERO; ADDRESS.tryte_offset.length])
    }
}

#[derive(Default, Debug, Clone)]
pub struct Value(pub i64);

impl Value {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct Tag(pub [Tryte; TAG.tryte_offset.length]);

impl Tag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(tag: &str) -> Self {
        assert!(tag.len() <= TAG.tryte_offset.length);
        assert!(tag.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; TAG.tryte_offset.length];

        for (i, c) in tag.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self([TRYTE_ZERO; TAG.tryte_offset.length])
    }
}

#[derive(Default, Debug, Clone)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Index(pub usize);

impl Index {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct Hash(pub [Tryte; BUNDLE_HASH.tryte_offset.length]);

impl Hash {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(hash: &str) -> Self {
        assert!(hash.len() <= BUNDLE_HASH.tryte_offset.length);
        assert!(hash.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; BUNDLE_HASH.tryte_offset.length];

        for (i, c) in hash.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self([TRYTE_ZERO; BUNDLE_HASH.tryte_offset.length])
    }
}

pub struct Nonce(pub [Tryte; NONCE.tryte_offset.length]);

impl Nonce {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(nonce: &str) -> Self {
        assert!(nonce.len() <= NONCE.tryte_offset.length);
        assert!(nonce.chars().all(|c| c.is_tryte()));

        let mut trytes = [TRYTE_ZERO; NONCE.tryte_offset.length];

        for (i, c) in nonce.chars().enumerate() {
            trytes[i] = c;
        }

        Self(trytes)
    }
}

impl Default for Nonce {
    fn default() -> Self {
        Self([TRYTE_ZERO; NONCE.tryte_offset.length])
    }
}

implement_debug!(Payload, Address, Tag, Nonce, Hash);
implement_display!(Payload, Address, Tag, Nonce, Hash);
implement_hash!(Address, Hash);
implement_eq!(Payload, Address, Tag, Hash, Nonce);
implement_clone!(Payload, Address, Tag, Nonce, Hash);

/// The (bundle) essence of each transaction is a subset of its fields, with a total size of 486 trits.
pub struct Essence<'a> {
    address: &'a Address,
    value: &'a Value,
    obsolete_tag: &'a Tag,
    timestamp: &'a Timestamp,
    index: &'a Index,
    last_index: &'a Index,
}

#[derive(Clone)]
pub struct Transaction {
    payload: Payload,
    address: Address,
    value: Value,
    obsolete_tag: Tag,
    timestamp: Timestamp,
    index: Index,
    last_index: Index,
    bundle: Hash,
    trunk: Hash,
    branch: Hash,
    tag: Tag,
    attachment_ts: Timestamp,
    attachment_lbts: Timestamp,
    attachment_ubts: Timestamp,
    nonce: Nonce,
}

impl Transaction {
    pub fn from_tryte_str(tx_trytes: &str) -> Result<Self> {
        if tx_trytes.len() != TRANSACTION_TRYT_LEN {
            return Err(Errors::TransactionDeserializationError);
        }
        unimplemented!()
    }
    /// Create a `Transaction` from a reader object.
    pub fn from_reader<R: std::io::Read>(reader: R) -> Result<Self> {
        unimplemented!()
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn obsolete_tag(&self) -> &Tag {
        &self.obsolete_tag
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn last_index(&self) -> &Index {
        &self.last_index
    }

    pub fn bundle(&self) -> &Hash {
        &self.bundle
    }

    pub fn trunk(&self) -> &Hash {
        &self.trunk
    }

    pub fn branch(&self) -> &Hash {
        &self.branch
    }

    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    pub fn attachment_ts(&self) -> &Timestamp {
        &self.attachment_ts
    }

    pub fn attachment_lbts(&self) -> &Timestamp {
        &self.attachment_lbts
    }

    pub fn attachment_ubts(&self) -> &Timestamp {
        &self.attachment_ubts
    }

    pub fn nonce(&self) -> &Nonce {
        &self.nonce
    }

    /// Returns the (bundle) essence of that transaction.
    pub fn essence<'a>(&'a self) -> Essence<'a> {
        Essence {
            address: &self.address,
            value: &self.value,
            obsolete_tag: &self.obsolete_tag,
            timestamp: &self.timestamp,
            index: &self.index,
            last_index: &self.last_index,
        }
    }

    pub fn builder() -> TransactionBuilder {
        TransactionBuilder::new()
    }
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "address={:?}\nvalue={:?}\ntimestamp={:?}\nindex={:?}\nlast_index={:?}\ntag={:?}\nbundle={:?}\ntrunk={:?}\nbranch={:?}\nnonce={:?}",
        self.address,
        self.value,
        self.timestamp,
        self.index,
        self.last_index,
        self.tag,
        self.bundle,
        self.trunk,
        self.branch,
        self.nonce)
    }
}

#[derive(Default)]
pub struct Transactions(pub(crate) Vec<Transaction>);

impl Transactions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, transaction: Transaction) {
        self.0.push(transaction);
    }
}

pub struct TransactionBuilder {
    payload: Option<Payload>,
    address: Option<Address>,
    value: Option<Value>,
    obsolete_tag: Option<Tag>,
    timestamp: Option<Timestamp>,
    index: Option<Index>,
    last_index: Option<Index>,
    bundle: Option<Hash>,
    trunk: Option<Hash>,
    branch: Option<Hash>,
    tag: Option<Tag>,
    attachment_ts: Option<Timestamp>,
    attachment_lbts: Option<Timestamp>,
    attachment_ubts: Option<Timestamp>,
    nonce: Option<Nonce>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            payload: None,
            address: None,
            value: None,
            obsolete_tag: None,
            timestamp: None,
            index: None,
            last_index: None,
            tag: None,
            bundle: None,
            trunk: None,
            branch: None,
            attachment_ts: None,
            attachment_lbts: None,
            attachment_ubts: None,
            nonce: None,
        }
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload.replace(payload);
        self
    }

    pub fn with_address(mut self, address: Address) -> Self {
        self.address.replace(address);
        self
    }

    pub fn with_value(mut self, value: Value) -> Self {
        self.value.replace(value);
        self
    }

    pub fn with_obsolete_tag(mut self, obsolete_tag: Tag) -> Self {
        self.obsolete_tag.replace(obsolete_tag);
        self
    }
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp.replace(timestamp);
        self
    }

    pub fn with_index(mut self, index: Index) -> Self {
        self.index.replace(index);
        self
    }

    pub fn with_last_index(mut self, last_index: Index) -> Self {
        self.last_index.replace(last_index);
        self
    }

    pub fn with_tag(mut self, tag: Tag) -> Self {
        self.tag.replace(tag);
        self
    }

    pub fn with_attachment_ts(mut self, attachment_ts: Timestamp) -> Self {
        self.attachment_ts.replace(attachment_ts);
        self
    }

    pub fn with_bundle(mut self, bundle: Hash) -> Self {
        self.bundle.replace(bundle);
        self
    }

    pub fn with_trunk(mut self, trunk: Hash) -> Self {
        self.trunk.replace(trunk);
        self
    }

    pub fn with_branch(mut self, branch: Hash) -> Self {
        self.branch.replace(branch);
        self
    }

    pub fn with_attachment_lbts(mut self, attachment_lbts: Timestamp) -> Self {
        self.attachment_lbts.replace(attachment_lbts);
        self
    }

    pub fn with_attachment_ubts(mut self, attachment_ubts: Timestamp) -> Self {
        self.attachment_ubts.replace(attachment_ubts);
        self
    }

    pub fn with_nonce(mut self, nonce: Nonce) -> Self {
        self.nonce.replace(nonce);
        self
    }

    /// Tries to build a transaction from the current state of the builder. If mandatory fields have not
    /// been set, this method will return a `TransactionBuilderError` describing which field has not been set.
    pub fn try_build(self) -> Result<Transaction> {
        Ok(Transaction {
            payload: self.payload.unwrap_or(Payload::new()),
            address: self.address.unwrap_or(Address::new()),
            value: self.value.unwrap_or(Value::new()),
            obsolete_tag: self.obsolete_tag.unwrap_or(Tag::new()),
            timestamp: self.timestamp.ok_or(Errors::TransactionBuilderError("timestamp not set"))?,
            index: self.index.unwrap_or(Index::new()),
            last_index: self.last_index.unwrap_or(Index::new()),
            tag: self.tag.unwrap_or(Tag::new()),
            bundle: self.bundle.ok_or(Errors::TransactionBuilderError("bundle hash not set"))?,
            trunk: self.trunk.ok_or(Errors::TransactionBuilderError("trunk hash not set"))?,
            branch: self.branch.ok_or(Errors::TransactionBuilderError("branch hash not set"))?,
            attachment_ts: self.attachment_ts.ok_or(Errors::TransactionBuilderError("attachment timestamp not set"))?,
            attachment_lbts: self.attachment_lbts.ok_or(Errors::TransactionBuilderError("attachment lower bound timestamp not set"))?,
            attachment_ubts: self.attachment_ubts.ok_or(Errors::TransactionBuilderError("attachment upper bound timestamp not set"))?,
            nonce: self.nonce.ok_or(Errors::TransactionBuilderError("nonce not set"))?,

        })
    }

    /// Builds a transaction from the current state of the builder. Even mandatory fields will be set to some
    /// default, hence this operation will always succeed even if the built transaction is certain to get rejected by
    /// the network.
    pub fn build_or_default(self) -> Transaction {
        Transaction {
            payload: self.payload.unwrap_or(Payload::new()),
            address: self.address.unwrap_or(Address::new()),
            value: self.value.unwrap_or(Value::new()),
            obsolete_tag: self.obsolete_tag.unwrap_or(Tag::new()),
            timestamp: self.timestamp.unwrap_or(Timestamp::new()),
            index: self.index.unwrap_or(Index::new()),
            last_index: self.last_index.unwrap_or(Index::new()),
            tag: self.tag.unwrap_or(Tag::new()),
            bundle: self.bundle.unwrap_or(Hash::new()),
            trunk: self.trunk.unwrap_or(Hash::new()),
            branch: self.branch.unwrap_or(Hash::new()),
            attachment_ts: self.attachment_ts.unwrap_or(Timestamp::new()),
            attachment_lbts: self.attachment_lbts.unwrap_or(Timestamp::new()),
            attachment_ubts: self.attachment_ubts.unwrap_or(Timestamp::new()),
            nonce: self.nonce.unwrap_or(Nonce::new()),
        }
    }
}

#[derive(Default)]
pub struct TransactionBuilders(pub(crate) Vec<TransactionBuilder>);

impl TransactionBuilders {
    pub fn push(&mut self, transaction_builder: TransactionBuilder) {
        self.0.push(transaction_builder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_transaction_from_builder() {
        let tx = Transaction::builder()
            .with_value(Value(10))
            .with_address(Address::from_str("ME"))
            .with_tag(Tag::from_str("HELLO"))
            .with_nonce(Nonce::from_str("ABCDEF"))
            .build_or_default();

        println!("{:?}", tx);
    }

    #[test]
    fn create_transaction_from_tryte_string() {
        //let tx = Transaction::from_tryte_str(TX_TRYTES);
    }

    const TX_TRYTES: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9BAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYOGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";
}
