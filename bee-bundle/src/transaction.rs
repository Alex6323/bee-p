use common::Tryte;
use common::constants::*;

use ternary::IsTryte;

use crate::constants::*;

pub struct Payload(pub(self) [Tryte; PAYLOAD.tryte_offset.length]);
pub struct Address(pub(self) [Tryte; ADDRESS.tryte_offset.length]);
#[derive(Default, Debug)]
pub struct Value(pub(self) i64);
pub struct Tag(pub(self) [Tryte; TAG.tryte_offset.length]);
#[derive(Default, Debug)]
pub struct Timestamp(pub(self) u64);
#[derive(Default, Debug)]
pub struct Index(pub(self) usize);
pub struct Hash(pub(self) [Tryte; BUNDLE_HASH.tryte_offset.length]);
pub struct Nonce(pub(self) [Tryte; NONCE.tryte_offset.length]);

#[derive(Default)]
pub struct Transaction {
    pub payload: Payload,
    pub address: Address,
    pub value: Value,
    pub obsolete_tag: Tag,
    pub timestamp: Timestamp,
    pub index: Index,
    pub last_index: Index,
    pub bundle_hash: Hash,
    pub trunk_hash: Hash,
    pub branch_hash: Hash,
    pub tag: Tag,
    pub attachment_ts: Timestamp,
    pub attachment_lbts: Timestamp,
    pub attachment_ubts: Timestamp,
    pub nonce: Nonce,
}

pub struct TransactionBuilder(Transaction);

impl Default for Payload {
    fn default() -> Self {
        Self([TRYTE_ZERO; PAYLOAD.tryte_offset.length])
    }
}

impl std::fmt::Debug for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().collect::<String>())
    }
}

impl Default for Address {
    fn default() -> Self {
        Self([TRYTE_ZERO; ADDRESS.tryte_offset.length])
    }
}

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().collect::<String>())
    }
}

impl Address {
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

impl Default for Tag {
    fn default() -> Self {
        Self([TRYTE_ZERO; TAG.tryte_offset.length])
    }
}

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().collect::<String>())
    }
}

impl Tag {
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

impl Default for Hash {
    fn default() -> Self {
        Self([TRYTE_ZERO; BUNDLE_HASH.tryte_offset.length])
    }
}

impl std::fmt::Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().collect::<String>())
    }
}

impl Default for Nonce {
    fn default() -> Self {
        Self([TRYTE_ZERO; NONCE.tryte_offset.length])
    }
}

impl std::fmt::Debug for Nonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().collect::<String>())
    }
}

impl Nonce {
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

impl Transaction {
    pub fn from_tryte_str(tx_trytes: &str) -> Self {
        assert_eq!(TRANSACTION_TRYT_LEN, tx_trytes.len());

        /*
        let payload = Payload::from_tx_tryte_str(tx_trytes, PAYLOAD.tryte_offset);
        
        Self {
            payload,
        }
        */

        unimplemented!()
    }
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "address={:?}\nvalue={:?}\ntimestamp={:?}\nindex={:?}\nlast_index={:?}\ntag={:?}\nbundle_hash={:?}\ntrunk_hash={:?}\nbranch_hash={:?}\nnonce={:?}", 
        self.address, 
        self.value, 
        self.timestamp, 
        self.index, 
        self.last_index, 
        self.tag, 
        self.bundle_hash, 
        self.trunk_hash, 
        self.branch_hash, 
        self.nonce)
    }
}

impl TransactionBuilder {
    pub fn default() -> Self {
        Self(Transaction::default())
    }

    pub fn payload(&mut self, payload: Payload) -> &mut Self {
        self.0.payload = payload;
        self
    }

    pub fn address(&mut self, address: Address) -> &mut Self {
        self.0.address = address;
        self
    }

    pub fn value(&mut self, value: Value) -> &mut Self {
        self.0.value = value;
        self
    }

    pub fn obsolete_tag(&mut self, obsolete_tag: Tag) -> &mut Self {
        self.0.obsolete_tag = obsolete_tag;
        self
    }
    pub fn timestamp(&mut self, timestamp: Timestamp) -> &mut Self {
        self.0.timestamp = timestamp;
        self
    }

    pub fn index(&mut self, index: Index) -> &mut Self {
        self.0.index = index;
        self
    }

    pub fn last_index(&mut self, last_index: Index) -> &mut Self {
        self.0.last_index = last_index;
        self
    }

    pub fn tag(&mut self, tag: Tag) -> &mut Self {
        self.0.tag = tag;
        self
    }

    pub fn attachment_ts(&mut self, attachment_ts: Timestamp) -> &mut Self {
        self.0.attachment_ts = attachment_ts;
        self
    }

    pub fn attachment_lbts(&mut self, attachment_lbts: Timestamp) -> &mut Self {
        self.0.attachment_lbts = attachment_lbts;
        self
    }

    pub fn attachment_ubts(&mut self, attachment_ubts: Timestamp) -> &mut Self {
        self.0.attachment_ubts = attachment_ubts;
        self
    }

    pub fn nonce(&mut self, nonce: Nonce) -> &mut Self {
        self.0.nonce = nonce;
        self
    }

    pub fn build(self) -> Transaction {
        self.0
    }
}

#[cfg(test)]
mod should {
    use super::*;

    #[test]
    fn create_transaction_from_builder() {
        let mut builder = TransactionBuilder::default();
        builder
            .value(Value(10))
            .address(Address::from_str("ME"))
            .tag(Tag::from_str("HELLO"))
            .nonce(Nonce::from_str("ABCDEF"));

        let tx = builder.build();
        //println!("{:?}", tx);
    }

    #[test]
    fn create_transaction_from_tryte_string() {
        let tx = Transaction::from_tryte_str(TX_TRYTES);
        
    }

    const TX_TRYTES: &str = "SEGQSWYCJHRLJYEGZLRYQAZPLVRAYIWGWJUMFFX99UZUKBQNFYAOQLOFARIKNEBKDRHJJWDJARXTNPHPAODJRSGJBVVYBVJHZALJWDCJHZRSACOVCVVAVHZVTPFTAJWVGFSVLSYXHNNXEGSMJHDBZKGFQNYJJJBAPDHFFGZ9POSOMWTDPGXI9KQRLMUVWNEQDANMXROVORJVALWVGDDJAFOOBXUKVCCIVXSSHZUCZV9XVBASLWX9NXPWGMGYCRD9ILQMKIGPBGGMKAIJKNALBLABATYFVIRBKTXTWNUZAUXRASB9EEIQHWBD9ZYUDBUPBSWXVYXQXECRCHQAYH9ZBUZBASPOIGBSGWJYFKFRITUBVMCYGCMAPTXOIWEVTUXSUOUPTUQOPMMPUTHXMOP9CW9THAZXEPMOMNEOBLUBPOAIOBEBERRZCIKHSTDWUSUPUWNJOCLNZDCEKWWAAJDPJXJEHHSYFN9MH9BGUDQ9CSZBIHRC9PSQJPGKH9ILZDWUWLEKWFKUFFFIMOQKRMKOYXEJHXLCEGCGGKHGJUHOXINSWCKRNMUNAJDCVLZGEBII9ASTYFTDYDZIZSNHIWHSQ9HODQMVNDKMKHCFDXIIGDIVJSBOOE9GRIXCD9ZUTWCUDKFTETSYSRBQABXCXZFOWQMQFXHYZWD9JZXUWHILMRNWXSGUMIIXZYCTWWHCWMSSTCNSQXQXMQPTM9MOQMIVDYNNARDCVNQEDTBKWOIOSKPKPOZHJGJJGNYWQWUWAZMBZJ9XEJMRVRYFQPJ9NOIIXEGIKMMN9DXYQUILRSCSJDIDN9DCTFGQIYWROZQIEQTKMRVLGGDGA9UVZPNRGSVTZYAPMWFUWDEUULSEEGAGITPJQ9DBEYEN9NVJPUWZTOTJHEQIXAPDOICBNNCJVDNM9YRNXMMPCOYHJDUFNCYTZGRCBZKOLHHUK9VOZWHEYQND9WUHDNGFTAS99MRCAU9QOYVUZKTIBDNAAPNEZBQPIRUFUMAWVTCXSXQQIYQPRFDUXCLJNMEIKVAINVCCZROEWEX9XVRM9IHLHQCKC9VLK9ZZWFBJUZKGJCSOPQPFVVAUDLKFJIJKMLZXFBMXLMWRSNDXRMMDLE9VBPUZB9SVLTMHA9DDDANOKIPY9ULDWAKOUDFEDHZDKMU9VMHUSFG9HRGZAZULEJJTEH9SLQDOMZTLVMBCXVNQPNKXRLBOUCCSBZRJCZIUFTFBKFVLKRBPDKLRLZSMMIQNMOZYFBGQFKUJYIJULGMVNFYJWPKPTSMYUHSUEXIPPPPPJTMDQLFFSFJFEPNUBDEDDBPGAOEJGQTHIWISLRDAABO9H9CSIAXPPJYCRFRCIH9TVBZKTCK9SPQZUYMUOKMZYOMPRHRGF9UAKZTZZG9VVVTIHMSNDREUOUOSLKUHTNFXTNSJVPVWCQXUDIMJIAMBPXUGBNDTBYPKYQYJJCDJSCTTWHOJKORLHGKRJMDCMRHSXHHMQBFJWZWHNUHZLYOAFQTRZFXDBYASYKWEVHKYDTJIAUKNCCEPSW9RITZXBOFKBAQOWHKTALQSCHARLUUGXISDMBVEUKOVXTKTEVKLGYVYHPNYWKNLCVETWIHHVTBWT9UPMTQWBZPRPRSISUBIBECVDNIZQULAGLONGVFLVZPBMHJND9CEVIXSYGFZAGGN9MQYOAKMENSEOGCUNKEJTDLEDCD9LGKYANHMZFSSDDZJKTKUJSFL9GYFDICTPJEPDSBXDQTARJQEWUVWDWSQPKIHPJONKHESSQH9FNQEO9WUCFDWPPPTIQPWCVDYTTWPLCJJVYNKE9ZEJNQBEJBMDBLNJKQDOQOHVS9VY9UPSU9KZVDFOESHNRRWBK9EZCYALAUYFGPCEWJQDXFENSNQEAUWDXJGOMCLQUQWMCPHOBZZ9SZJ9KZXSHDLPHPNYMVUJQSQETTN9SG9SIANJHWUYQXZXAJLYHCZYRGITZYQLAAYDVQVNKCDIYWAYBAFBMAYEAEAGMTJGJRSNHBHCEVIQRXEFVWJWOPU9FPDOWIFL9EWGHICRBNRITJDZNYACOGTUDBZYIYZZWAOCDBQFFNTTSTGKECWTVWZSPHX9HNRUYEAEWXENEIDLVVFMZFVPUNHMQPAIOKVIBDIHQIHFGRJOHHONPLGBSJUD9HHDTQQUZN9NVJYOAUMXMMOCNUFLZ9BAJSZMDMPQHPWSFVWOJQDPHV9DYSQPIBL9LYZHQKKOVF9TFVTTXQEUWFQSLGLVTGK99VSUEDXIBIWCQHDQQSQLDHZ9999999999999999999TRINITY99999999999999999999TNXSQ9D99A99999999B99999999MXKZAGDGKVADXOVCAXEQYZGOGQKDLKIUPYXIL9PXYBQXGYDEGNXTFURSWQYLJDFKEV9VVBBQLTLHIBTFYOGBHPUUHS9CKWSAPIMDIRNSUJ9CFPGKTUFAGQYVMFKOZSVAHIFJXWCFBZLICUWF9GNDZWCOWDUIIZ9999OXNRVXLBKJXEZMVABR9UQBVSTBDFSAJVRRNFEJRL9UFTOFPJHQMQKAJHDBIQAETS9OUVTQ9DSPAOZ9999TRINITY99999999999999999999LPZYMWQME999999999MMMMMMMMMDTIZE9999999999999999999999";
}