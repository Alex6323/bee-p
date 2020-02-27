use crate::constants::IOTA_SUPPLY;

use bee_common::constants::{
    ADDRESS_TRIT_LEN, HASH_TRIT_LEN, NONCE_TRIT_LEN, PAYLOAD_TRIT_LEN, TAG_TRIT_LEN,
    TRANSACTION_TRYT_LEN,
};
use bee_common::Errors;
use bee_common::Result as BeeResult;
use bee_ternary::{raw::RawEncodingBuf, util::trytes_to_trits_buf, IsTryte, T1B1Buf, TritBuf};

use std::fmt;

macro_rules! implement_debug {
    ($($t:ty),+) => {
    $(
        impl<T: RawEncodingBuf> fmt::Debug for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
    )+
    }
}

macro_rules! implement_display {
    ($($t:ty),+) => {
    $(
        impl<T: RawEncodingBuf> fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
    )+
    }
}

macro_rules! implement_eq {
    ($($t:ty),+) => {
    $(
        impl<T: RawEncodingBuf> PartialEq for $t {
            fn eq(&self,other: &$t) -> bool {
                    self.0.iter().zip(other.0.iter()).all(|(a,b)| a == b)
            }
        }

        impl<T: RawEncodingBuf> Eq for $t {}

    )+
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Payload<T: RawEncodingBuf = T1B1Buf>(TritBuf<T>);

impl Payload {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(PAYLOAD_TRIT_LEN))
    }

    pub fn from_str(payload: &str) -> Self {
        assert!(payload.len() <= PAYLOAD_TRIT_LEN / 3);
        assert!(payload.chars().all(|c| c.is_tryte()));

        let missing_trytes = PAYLOAD_TRIT_LEN / 3 - payload.len();
        let payload = payload.to_owned() + &"9".repeat(missing_trytes);
        let tritbuf = trytes_to_trits_buf(&payload);

        Self(tritbuf)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Address<T: RawEncodingBuf = T1B1Buf>(TritBuf<T>);

// TODO Hash

impl Address {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(ADDRESS_TRIT_LEN))
    }

    pub fn from_str(address: &str) -> Self {
        assert!(address.len() <= ADDRESS_TRIT_LEN / 3);
        assert!(address.chars().all(|c| c.is_tryte()));

        let missing_trytes = ADDRESS_TRIT_LEN / 3 - address.len();
        let address = address.to_owned() + &"9".repeat(missing_trytes);
        let tritbuf = trytes_to_trits_buf(&address);

        Self(tritbuf)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value(pub i64);

#[derive(Clone, Serialize, Deserialize)]
pub struct Tag<T: RawEncodingBuf = T1B1Buf>(TritBuf<T>);

impl Tag {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(TAG_TRIT_LEN))
    }

    pub fn from_str(tag: &str) -> Self {
        assert!(tag.len() <= TAG_TRIT_LEN / 3);
        assert!(tag.chars().all(|c| c.is_tryte()));

        let missing_trytes = TAG_TRIT_LEN / 3 - tag.len();
        let tag = tag.to_owned() + &"9".repeat(missing_trytes);
        let tritbuf = trytes_to_trits_buf(&tag);

        Self(tritbuf)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index(pub usize);

#[derive(Clone, Serialize, Deserialize)]
pub struct Hash<T: RawEncodingBuf = T1B1Buf>(TritBuf<T>);

// TODO Hash

impl Hash {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(HASH_TRIT_LEN))
    }

    pub fn from_str(hash: &str) -> Self {
        assert!(hash.len() <= HASH_TRIT_LEN / 3);
        assert!(hash.chars().all(|c| c.is_tryte()));

        let missing_trytes = HASH_TRIT_LEN / 3 - hash.len();
        let hash = hash.to_owned() + &"9".repeat(missing_trytes);
        let tritbuf = trytes_to_trits_buf(&hash);

        Self(tritbuf)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Nonce<T: RawEncodingBuf = T1B1Buf>(TritBuf<T>);

impl Nonce {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(NONCE_TRIT_LEN))
    }

    pub fn from_str(nonce: &str) -> Self {
        assert!(nonce.len() <= NONCE_TRIT_LEN / 3);
        assert!(nonce.chars().all(|c| c.is_tryte()));

        let missing_trytes = NONCE_TRIT_LEN / 3 - nonce.len();
        let nonce = nonce.to_owned() + &"9".repeat(missing_trytes);
        let tritbuf = trytes_to_trits_buf(&nonce);

        Self(tritbuf)
    }
}

implement_debug!(Payload<T>, Address<T>, Tag<T>, Nonce<T>, Hash<T>);
implement_display!(Payload<T>, Address<T>, Tag<T>, Nonce<T>, Hash<T>);
implement_eq!(Payload<T>, Address<T>, Tag<T>, Nonce<T>, Hash<T>);

/// The (bundle) essence of each transaction is a subset of its fields, with a total size of 486 trits.
pub struct Essence<'a, T: RawEncodingBuf = T1B1Buf> {
    address: &'a Address<T>,
    value: &'a Value,
    obsolete_tag: &'a Tag<T>,
    timestamp: &'a Timestamp,
    index: &'a Index,
    last_index: &'a Index,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction<T: RawEncodingBuf = T1B1Buf> {
    pub(crate) payload: Payload<T>,
    pub(crate) address: Address<T>,
    pub(crate) value: Value,
    pub(crate) obsolete_tag: Tag<T>,
    pub(crate) timestamp: Timestamp,
    pub(crate) index: Index,
    pub(crate) last_index: Index,
    pub(crate) bundle: Hash<T>,
    pub(crate) trunk: Hash<T>,
    pub(crate) branch: Hash<T>,
    pub(crate) tag: Tag<T>,
    pub(crate) attachment_ts: Timestamp,
    pub(crate) attachment_lbts: Timestamp,
    pub(crate) attachment_ubts: Timestamp,
    pub(crate) nonce: Nonce<T>,
}

impl<T: RawEncodingBuf> Transaction<T> {
    pub fn from_tryte_str(tx_trytes: &str) -> BeeResult<Self> {
        if tx_trytes.len() != TRANSACTION_TRYT_LEN {
            return Err(Errors::TransactionDeserializationError);
        }
        unimplemented!()
    }

    /// Create a `Transaction` from a reader object.
    pub fn from_reader<R: std::io::Read>(reader: R) -> BeeResult<Self> {
        unimplemented!()
    }

    pub fn payload(&self) -> &Payload<T> {
        &self.payload
    }

    pub fn address(&self) -> &Address<T> {
        &self.address
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn obsolete_tag(&self) -> &Tag<T> {
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

    pub fn bundle(&self) -> &Hash<T> {
        &self.bundle
    }

    pub fn trunk(&self) -> &Hash<T> {
        &self.trunk
    }

    pub fn branch(&self) -> &Hash<T> {
        &self.branch
    }

    pub fn tag(&self) -> &Tag<T> {
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

    pub fn nonce(&self) -> &Nonce<T> {
        &self.nonce
    }

    /// Returns the (bundle) essence of that transaction.
    pub fn essence<'a>(&'a self) -> Essence<'a, T> {
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

impl<T: RawEncodingBuf> fmt::Debug for Transaction<T> {
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
pub struct Transactions<T: RawEncodingBuf = T1B1Buf>(pub(crate) Vec<Transaction<T>>);

impl<T: RawEncodingBuf> Transactions<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get(&self, index: usize) -> Option<&Transaction<T>> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, transaction: Transaction<T>) {
        self.0.push(transaction);
    }
}

/// Transaction builder

#[derive(Debug)]
pub enum TransactionBuilderError {
    MissingField(&'static str),
    InvalidValue(i64),
}

pub struct TransactionBuilder<T: RawEncodingBuf = T1B1Buf> {
    pub(crate) payload: Option<Payload<T>>,
    pub(crate) address: Option<Address<T>>,
    pub(crate) value: Option<Value>,
    pub(crate) obsolete_tag: Option<Tag<T>>,
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) index: Option<Index>,
    pub(crate) last_index: Option<Index>,
    pub(crate) bundle: Option<Hash<T>>,
    pub(crate) trunk: Option<Hash<T>>,
    pub(crate) branch: Option<Hash<T>>,
    pub(crate) tag: Option<Tag<T>>,
    pub(crate) attachment_ts: Option<Timestamp>,
    pub(crate) attachment_lbts: Option<Timestamp>,
    pub(crate) attachment_ubts: Option<Timestamp>,
    pub(crate) nonce: Option<Nonce<T>>,
}

impl<T: RawEncodingBuf> Default for TransactionBuilder<T> {
    fn default() -> Self {
        Self {
            payload: None,
            address: None,
            value: None,
            obsolete_tag: None,
            timestamp: None,
            index: None,
            last_index: None,
            bundle: None,
            trunk: None,
            branch: None,
            tag: None,
            attachment_ts: None,
            attachment_lbts: None,
            attachment_ubts: None,
            nonce: None,
        }
    }
}

impl<T: RawEncodingBuf> TransactionBuilder<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_payload(mut self, payload: Payload<T>) -> Self {
        self.payload.replace(payload);
        self
    }

    pub fn with_address(mut self, address: Address<T>) -> Self {
        self.address.replace(address);
        self
    }

    pub fn with_value(mut self, value: Value) -> Self {
        self.value.replace(value);
        self
    }

    pub fn with_obsolete_tag(mut self, obsolete_tag: Tag<T>) -> Self {
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

    pub fn with_tag(mut self, tag: Tag<T>) -> Self {
        self.tag.replace(tag);
        self
    }

    pub fn with_attachment_ts(mut self, attachment_ts: Timestamp) -> Self {
        self.attachment_ts.replace(attachment_ts);
        self
    }

    pub fn with_bundle(mut self, bundle: Hash<T>) -> Self {
        self.bundle.replace(bundle);
        self
    }

    pub fn with_trunk(mut self, trunk: Hash<T>) -> Self {
        self.trunk.replace(trunk);
        self
    }

    pub fn with_branch(mut self, branch: Hash<T>) -> Self {
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

    pub fn with_nonce(mut self, nonce: Nonce<T>) -> Self {
        self.nonce.replace(nonce);
        self
    }

    pub fn build(self) -> Result<Transaction<T>, TransactionBuilderError> {
        let value = self
            .value
            .as_ref()
            .ok_or(TransactionBuilderError::MissingField("value"))?
            .0;

        if value.abs() > IOTA_SUPPLY {
            return Err(TransactionBuilderError::InvalidValue(value));
        }

        Ok(Transaction::<T> {
            payload: self
                .payload
                .ok_or(TransactionBuilderError::MissingField("payload"))?,
            address: self
                .address
                .ok_or(TransactionBuilderError::MissingField("address"))?,
            value: Value(value),
            obsolete_tag: self
                .obsolete_tag
                .ok_or(TransactionBuilderError::MissingField("obsolete_tag"))?,
            timestamp: self
                .timestamp
                .ok_or(TransactionBuilderError::MissingField("timestamp"))?,
            index: self
                .index
                .ok_or(TransactionBuilderError::MissingField("index"))?,
            last_index: self
                .last_index
                .ok_or(TransactionBuilderError::MissingField("last_index"))?,
            tag: self
                .tag
                .ok_or(TransactionBuilderError::MissingField("tag"))?,
            bundle: self
                .bundle
                .ok_or(TransactionBuilderError::MissingField("bundle"))?,
            trunk: self
                .trunk
                .ok_or(TransactionBuilderError::MissingField("trunk"))?,
            branch: self
                .branch
                .ok_or(TransactionBuilderError::MissingField("branch"))?,
            attachment_ts: self
                .attachment_ts
                .ok_or(TransactionBuilderError::MissingField("attachment_ts"))?,
            attachment_lbts: self
                .attachment_lbts
                .ok_or(TransactionBuilderError::MissingField("attachment_lbts"))?,
            attachment_ubts: self
                .attachment_ubts
                .ok_or(TransactionBuilderError::MissingField("attachment_ubts"))?,
            nonce: self
                .nonce
                .ok_or(TransactionBuilderError::MissingField("nonce"))?,
        })
    }
}

#[derive(Default)]
pub struct TransactionBuilders(pub(crate) Vec<TransactionBuilder>);

impl TransactionBuilders {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, transaction_builder: TransactionBuilder) {
        self.0.push(transaction_builder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_transaction_from_builder() {
        let tx = TransactionBuilder::new()
            .with_payload(Payload::zeros())
            .with_address(Address::zeros())
            .with_value(Value(0))
            .with_obsolete_tag(Tag::zeros())
            .with_timestamp(Timestamp(0))
            .with_index(Index(0))
            .with_last_index(Index(0))
            .with_tag(Tag::zeros())
            .with_attachment_ts(Timestamp(0))
            .with_bundle(Hash::zeros())
            .with_trunk(Hash::zeros())
            .with_branch(Hash::zeros())
            .with_attachment_lbts(Timestamp(0))
            .with_attachment_ubts(Timestamp(0))
            .with_nonce(Nonce::zeros())
            .build()
            .unwrap();
    }

    #[test]
    fn create_transaction_from_tryte_string() {
        //let tx = Transaction::from_tryte_str(TX_TRYTES);
    }

    const TX_TRYTES: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9BAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYOGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";
}
