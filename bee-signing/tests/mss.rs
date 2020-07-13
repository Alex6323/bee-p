// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

#[cfg(test)]
mod tests {

    use bee_crypto::ternary::{CurlP27, CurlP81, Kerl, Sponge};
    use bee_signing::ternary::{
        MssError, MssPrivateKeyGeneratorBuilder, MssPublicKey, MssSignature, PrivateKey, PrivateKeyGenerator,
        PublicKey, RecoverableSignature, Seed, Signature, WotsPublicKey, WotsSecurityLevel,
        WotsSpongePrivateKeyGenerator, WotsSpongePrivateKeyGeneratorBuilder,
    };
    use bee_ternary::{T1B1Buf, TryteBuf};

    #[test]
    fn mss_generator_missing_depth() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        match MssPrivateKeyGeneratorBuilder::<Kerl, WotsSpongePrivateKeyGenerator<Kerl>>::default()
            .generator(wots_private_key_generator)
            .build()
        {
            Ok(_) => unreachable!(),
            Err(err) => assert_eq!(err, MssError::MissingDepth),
        }
    }

    #[test]
    fn mss_generator_invalid_depth() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        match MssPrivateKeyGeneratorBuilder::<Kerl, WotsSpongePrivateKeyGenerator<Kerl>>::default()
            .generator(wots_private_key_generator)
            .depth(21)
            .build()
        {
            Ok(_) => unreachable!(),
            Err(err) => assert_eq!(err, MssError::InvalidDepth(21)),
        }
    }

    #[test]
    fn mss_generator_missing_generator() {
        match MssPrivateKeyGeneratorBuilder::<Kerl, WotsSpongePrivateKeyGenerator<Kerl>>::default()
            .depth(5)
            .build()
        {
            Ok(_) => unreachable!(),
            Err(err) => assert_eq!(err, MssError::MissingGenerator),
        }
    }

    fn mss_wots_generic_signature_verify<S>(public_key: &str, message: &str, signature: &str, depth: u8, index: u64)
    where
        S: Sponge + Default,
    {
        let public_key_trits = TryteBuf::try_from_str(public_key)
            .unwrap()
            .as_trits()
            .encode::<T1B1Buf>();
        let message_trits = TryteBuf::try_from_str(message).unwrap().as_trits().encode::<T1B1Buf>();
        let signature_trits = TryteBuf::try_from_str(signature)
            .unwrap()
            .as_trits()
            .encode::<T1B1Buf>();

        let public_key = MssPublicKey::<S, WotsPublicKey<S>>::from_trits(public_key_trits).depth(depth);
        let signature = MssSignature::<S>::from_trits(signature_trits).index(index);
        let valid = public_key.verify(message_trits.as_i8_slice(), &signature).unwrap();

        assert!(valid);
    }

    #[test]
    fn mss_wots_kerl_sec_1_signature_verify() {
        const PUBLIC_KEY: &str = "ECRGOIGKMFCNJPILB9GRUN9WIFOXY9GPKLSJV9UUQINIOHWKYJRZEQ9IHTS9HMFCMQBGRNODBIWTPILGC";
        const MESSAGE: &str = "KEWPSJHHGOICFXVGNUNRUDSKDUKNWGADKUFOFYVTZVGBVLWGIQBOICNNZIMWAXMV9RRMWSYGIABIBZUZ9";
        const SIGNATURE: &str = "DJ9WGAKRZOMH9KVRCHGCDCREXZVDKY9FXAXVSLELYADXHQCQQSMQYAEEBTEIWTQDUZIOFSFLBQQA9RUPXZ9THOGJIBPKN9XLYMNYMWHWRKJMDTGM9BNFDZQH9IVIOGSOMNUBDAGUMWECIQX9YVLNIXPXYTAACLZRIYPFVDVVDGQXCROYORAPAWIOQGHRVWNBXKAAGBGGYOORIUDBMYYZXTCUHDYFDZOHOERAOP9VCNKEULPU9ZCVKOMSYNCOPXTKTTFLWHQKEVNIAKLPKJ9YHBZDBQZDSAJYGVVSKMQZ9OJNBDUDJTGAE9OLPACMMX9WLVC9TWJRRGNLQJPJMEPBAFBNYJ9CWMTLTWIHISDMKRMUUHSPLIXMWYIMBAUOFTKMQSSBZOXTKKBADHUJFCQVYJJCAUGAEJJPJWWFWCZNHRFZZLMQLDVRQAVNHJBOXGZCWQZCLKDIDXSAHYOCSOUIHMIRTM9PHPTTGWUCJWNYPSIGTRQSUWQMFCXAKNJZZUCMGQUDJWZJRSLTHPIPVLMKZRTDJBGHQOSUXAENIGPKRBSZPAPFAGNPBXXHZR9SEWVCRSGUXIJLKSAJVCJDBJXJY9M9LVLWJAMHHNGAIVZ9DFGOFGPHLVDSDEGPNJZWVNGHKWEUFQRCMEGNLJMAFJKIAZBRPUDZTFLGRLADIBRNNIGWAKCCZD9KCFRX9ENNXGR9MBCDFJWUCZUMPOFW9GWF9GRBDVHWOLSXVWDIVKBOARUPCZSLVD9UNSKLDLJGLPHZR9KFSSGLRCHMESLBYUDIJTGPARIZFQROWSRZOZAJW9TPYIA9YHECNANSTMNLBUWJZVLTYAILPGJLCUNGXQNGBRXCBFCONWEIDXD9OQIWNCJZDFXOAVOVOUMGTOYB9ATJLKPVRHWPHGZSBPMEUWNDZGVMZYVAFQT9YGBGXDBDCTLTHBJEYDOZPLAAZCMFKHAQ9CAXVGZUTJYDMPL9DIMAUATSIOSHWJWAVPWIQIMXOLAYEIUGDISTOCIGGPA9WNVOSDDXVPOMFZKNKMSNIMXKCTGPNCRO9NQFYXVRTPSXHRRDGHMKNLFEPKAXDZLOCSFCVMIBEIQMJTBHWUHXNH9WGSKVCRBDRXRWAURDMNSKFLGMPXQEBXKKENSUKRZMQZUMNHJYWEZXIRPEPSKPC9JRGNYWAJFPN9AZKGYZCHVBCHHPTLXOURSVUHCM9QDACGWBDVTMNZKNVTOJSINYRDYNRPCMEBDUMUXFRCPYRHXOHDFGJFXOETGT9TOJVNWBEYXSK9PVVYZAS9WOSWJAH9UFXFFNWKWEUKXZM9BJDRXFFTQYRIVKWVFKCMSRDVNWVDPXWT9KOEBIDUKBE9MNKENKVABUSUDMIKZLCSBS9ZDTZNXRKCMIQTDEHYKEBPJUIGFZNCTSGOCQAVZIBFXKCREXDLWRBUDXWPHPYEDFLMSUBODJPSUIHCAHGLQZZHRQCMHRCSTKWCRRJHEVYXKGKIXRXSFREXRKQHHOIETURROFUTKYIDSQFAKZYWJKZESEYXIPKXCJLIPFCORSVH9WKFHJLUAQ9NSTRYBIFZIWZXMV9ZCCPTELAFRAMRFCOQHNOTLAXVTKLHEILANKBOTOQDACLYWXXKVFDIODLYNUIXQXPRFZTELIBDJUXJFPIQ9FRTGEXHOLOIVKCVTGLGHQNTONFRLAHDLHVSWDKBNH9ILD9IKJPNCBNAOASMFZVLUSOMPAHNIXRAKQDNTUZSCVNUOORADATIBHLHXTAADAUTPFTQEIRGWHMNBGSWPGMHAXNEUWVPETBYQKKTLBKVCEDYUSZ9KYMWMRYKDYWMFKJYEGHUSGDIVZWFFFQXAHSJYTRIAVQNXG9WXRIZFCZPWMOXZQRFAFMVRUCDSACJ9FRATCGNOPJRSGGTOCJIZIQNYOFKZWHBCRKLERIECSBLQND9ATPIFCSNPONGMI9LMRMKDHIFEGYRWPHBYHLS9ZXVE9JLJNIMCTGVTTCWQOYPREFLBJUAVBOE9JKHTWHKZPKCMPAGPFIVRWIBEXNPTEKCNDZOIOHLVWEGWSEKRSW9DUIXDKPQLQNZWGFMMCTGYKZWYHNRLQRDTIDFZRBVOQFCRMURK9CZS9IZPUSCIEPPCCTFKBS9FYXSPIKTNWXYENQDUNXLJWVBJBSQTUHIGDFRWEJEXPZYZTJAPZR99TTEXIIPGTIFRSLZKAKAAPACWEKAIAWVQRODDVMHOGGMSXEDAACFPSTWKARTNLZCBDISLZSJPAGCRZEDQBWIMQIUASDWYTZRZMYGZDEQLWCIJY9ULUIYQWWIFKEBZAIINGPMWCEFYWXQAPCYNOTL9HMYDNQMUEJJVDAQ9HRHZMI9NRJWLFM9SQIOFYXDBGCCBEWDQWUHIGFZKHJNRWMFSEFAWPM9AYNEQVUDKPPLK9WPLDFQBHLWWWGRTL9QCFMMMFKIEAORYLUEHFZMMSVVUHVNEJVTWKNUVOLSFEIFSZIDGOKPUXJADTAKWCYZZVQE9LWEDDRRKEFDHUUUPVTZHGBTBAAY9EQYGTNUFETRJBPPUP9HBJHXTPEUWDFDACXRQCAKGLBIK9GPGCVHUMRW9CCKMKJIAZEAWP9GMVRWQFTGDRGHZHPRSJTBRRPJQHMIDFJKSKKVVZWASKJG9FVWUGARVWXRBUFBSWNHPPXAKKX9MBGZUBRYCXWVSWWAEMTFGMMLYMGXSAWWQNJCGAXDVSKZTHW9NFKJHAQTKAOWHXBTLIUDLCQX9GPW999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999";
        const DEPTH: u8 = 8;
        const INDEX: u64 = 0;

        mss_wots_generic_signature_verify::<Kerl>(PUBLIC_KEY, MESSAGE, SIGNATURE, DEPTH, INDEX);
    }

    #[test]
    fn mss_wots_kerl_sec_3_signature_verify() {
        const PUBLIC_KEY: &str = "IDSWNWLGPFLAQADAEYUINRS9MBEMCYARHXHVSBOZDOBHPIPNVYUFFTQLNYGDZKKTEBHYOQXVQVHXBGXH9";
        const MESSAGE: &str = "NNZLXQKRAQBEUKNGVTKAHIIJUGSNNNNCASGGPNBJHKGTH9EGEAJZPKYL9WTNVYHFKDSQYERI9AYUFHYB9";
        const SIGNATURE: &str = "PGCWBHJEXWRBTMSIWGIDRXWNFTYGTTXPEAHWEFXKXGH9VA9JARHWUHUYEOOYENBHKNF9WLIFOH9HGQJVDOMSJ9XWTOVUDVDJBDCSKOYHM9QQFEHHMGUMXNBTWVBDCBBCAGEAKCWMHUUGBIXURKNMUSQTWQVRYGQAS9SJFAXLWXALFRWJUFNS9LDMOVUFVPHBZPDLYD9DKSWDY9TUOCCQQM9JXMTJRLRWEUBAQLJCOYJTASG9QBXKKGDTRWHQGATGSWDEEZUU9EJQACNU9CHIUIQRGLWGHFF9ABGE9SGYFRANEOZJLRHCYRWI9IT9FGYMKYGYMYDWRUOFMDGIR9EYNMIHWDBAJRBBYBBRL9YAWDGWUVGWLMKKVJQAAFVOTYMQNBAGZZFLVP9HHENEFZC9DPLOABBV9FKJCYJ9OH9BTBNEQTJSJICBWHUQVHSOISVIIRCUUEVJLJNLZUGBQUDXTUNS9HQYXHNAQWTQIWYYERXQKFZBEJGRDJJLZUNNTZDDEIYZBFWFF9ENLLS9TNPFXJARPBMYFOAQEP9GYBRNAAFOPBRSTOKPCUZQLQAMNWHVKZHVPHDLDRORAUTLQKZNPTIAERPPMDD99WTVZGGJWCLVOWXIJHEKAKINBSQQEBFIWTFIZINBGEOI9IRNCVY9QBQECCDCTOTAOEUCLJCM9MSNXJMHHNFGIBLT9KDMASEGKFPOAQQ9ZFXUSRCNZTGPOXHDWTGUAL9JYLMOOKLWCTNC9UKMEFSBZSV9DXBYTOHNMUEAJTHJNVBMCYIEDOICPHROXUVMA9UREPMUCPBPJDELBJEBZYOR9D9TKLEQ9G9GXRXPSDBDZXTTUAKJSAIRCFTCTQFRLUDQBPRXIYCDAXCUOZIGZKCZRWXSLLZLAJTCDCEUIPYASTQOLMSEYEIMVBEGKOBAPYJZMHPGSXPFUGYLWGZZPBVCBKZBBCSPGEGBZAJQUCHSMSNW9HFEDYHOJAUJBJSKYFSMTWTTF9HGERJZIYMMLYBRVLJLCQFMXNXUPILZSERKSNMVTLMSKJZRIAVN9IRIBNBYLU9MW9EVPBEESKHOWTETUTZRHKM9BKWRFLZGIREKNC9KSKHBPRLCHCIEXPZEXILOYXBOPFLMWHSCN9JWUGZPWCWFSBNFFOKBQKPIJSUXNN9UZQQDKKEWZQDRXPDYQJHPBZJTOLQEFDWCKFPQELRFWTVBZVVZTAXDRQIGJVFNBM9TZCKRE9WDFTFVPV9RYIIUDIOIPYUG9UZCSVTJCLLSAFQJKPZGTTQLQYIAYGWJUMARKHSXKGCOXKKBBVUSPZZETMAGQBPAH9WUXLUJQGZKQUYUWMPCFCHMNGNQL9XNMFVEKMIPVTBXLBNKGDFVLLHNAREIC9HLZHNZYMBJHWAGESNDANOLPXHE9CVDQPVJBQHLTZLOEBSHZPMTAMLDTTNVYJWGOXCSGUEFIMLY9CNCLJSAUDLMRRFXSESMPMXDZRVTHZGLKJCHUYHNBNGBUOPISW9TAGGJY9FUQVLX9BRDJGRQ9BHCMLCZPJZFLCFBPDCIHAGJNNT9OKHSRBHPPNPNFFKYBUTXKUOTHH9ASBNDCYTUQOJWMYUDFDGBRPRSKKHQIOERZPWZWWZQLSORYDOU9QV9UW9NZFVBGUCFBNGGJSPYDVPXO9ZBDR9WYJIYDRNKIJQTJCJOOAEQT9OTADGBDAZACMTWKWFDMSS9BGLSNKZVFTKDQDNSGBACUVFCQBJXMVNPIY9RUR9IUWTDUJFQWTJTWUQYNSSZB9QWGDPXJWNUSKWUYBVIC9BSMNHQVKXSOUSJUBZDOUOMEXAX9KDEWZSWRJZNQDBCXQZFEZJMCMPMOA9ZVCHLQDUCMFDNIFCHOSIJIPCKTFOPVUTZCTZWAWBIGMQRRQGCNQYMAYZXDPXBFAKVDEPMJUXPMRLEMXUMFNSMGHZ9MSEWKYPWOUPSIOZTEJYUFLRVHXCMCTQX9IVNKMTQYQCHW9VSSSXJBOJPBGBIDIIKNRUQBDZUSOMOSWRZCGBLZZWNJQFRUWPBPXHMPVGOEZATZSDRLULE9ECFUMNPWELRL9AAJCONHTXMXSSIXNQBCNNLGREGDPRDWBNOVCOJVTKDPVRAYLXMFMZHABVZXZAIKUJGHYGYQWTBIDFHLBCLRASI9MTFFFHLJOYSIOSGPFKPUDYAXVRVXGDERBTZIAEG9UFXMWDGOCMXZSOHSCAPPYKWUJQLXJCIKJOP9NJJXRLCDEBUWMAKTNLVSJEZNTDVRPXWWKEZXIKBWEQRIR9RDIKXNPTIKNQWWOMIAECGNCYWY9LDSZQBVAOIJFWBYFYKXPLWPNWGDVMOVTNYMH9ZLYUBXPCOWOHJEHBTWEX9XNBWKOTGKWYCTIIKTZSUJPPXNXZOS9USOUIPOAZFWZRMQKKOWSSRMMTIVYQHRIODKXAKPH9FSJNQ9ZUEJNUMHPOPINIUZGNAIDBWEDSGPDPXRWLNKCMJCGYQBDWIGULQKYKLPHQQDDHBWNRVEBXJDSBGWSXEJFBYVVNOIOOQOTKLJYVDVRQUZCVKFBPROB9VVSHLAEKEEJLQHHNZDQLNAKEHBVPDSKGNHXTZNA9WUXBTK99SZXZYRHHHGRZIRLRQEVQSGPVHIOPKP9T9AGGJBTFASXWMKAOFUUZYMRHBUXRBZAO9SGMACE9CXLXCQADLITOI9AWXSUXUPAYFVNIMUZAGCJLZGCKX9IFJQKSNAAG9YBZRJBGJFWBSYCYV99GOSLLMSAXJNQWMCCQWBARLWDDRXUYBUIKHTAOGWWXLTZKOCBYIGBKVFCHDAPQIBBMJFPCQMQCWZYAMGWRBTMBJNDCXP9ADBXSUBAROQYODUWKDSXFMPKANYEJYESWNCHVDKCDBCQXXCRRENIIABYT9WUMDWRVRPSLLS99HYAACBKEWJQXXEWMHXFAEPVQD9KTDZFIWTGBLEBWAYAHVRNLGKPHXFNFLSFWRIKX99OTIUQVFYA9YPJAJJEYGXFSINSXDGNPUKPERLCCSCPHU9WKCGNWMCQRXBTWRRXACOVLKPWSHOXKLRIRWBDMEEUJOOALMYZLIWPHSCWXHV9RNHTFAUSPRHZMR9OAJWFWQENFOFCPMVJEPBQAS9LJWDUP9WKFDWOZRREWX9TSNCCDCHNMVBYGCIF9VVVBWRKDRGJNQQLIP9CABUZRISLUJIZTWIEXGDLMBVIFVEUWSDGTRVLGOPTEUOWUY99G9OEKUFIORYCXXQMTSL9MCVUQJWMVTSVSEMXREZHBIETUVNNUACCVFUECTCKWMUUCLXOEY9LQFAYGUKWGFZWXTTVCXVYJYCRCCARSKWMIGBZONSJIAACSNZUFBVBASXEXHXIJEQTEANIDZSREFIUOEVRSKUGFHMNV9ZQDUFBZAXNHZLZTH9YSVRWNEXAZVXPYIPRSNRHNZUJPALZQHUO9QLAS9RX9QBIXMAGIRNM9GUUYOGUBQOXXLKQEZPZMHKNCITFOYBEUKAXHJLRRMMVF9SLJZDGQAM9UXQ9VMVLIHFUVSNLBVVMDLYULLELHMTFTWCNUEXXQFDIJSNWUDWYLWAXEUSVOJFCCCYZFLQNFFRTPOTVVUALPYZCTUSEQOKVHCTZETMDMFSNSNGDTUCEGJYYZTXGRMUZJTWCULIZWUIBOVJAISLCGJ9MFOJIDZROGYYXHEXAIIWOAVEJWXVVQEKZXVIT99LBOIKGZWNX9OPTPLSOQPEAOIRKYIMLXHYLPVPXG9AKFIOFBFEBLASXYENQ9WXIMKILNNJOAJHWODVRBGNCBGWT9BHZZYDPJXQQHSO9CNDSPNZZEIPVIMNAPVCSXTTFGJSRVDYTMIQAKFPUEZAT9AKAQDBMP9FKXLQZUDDSMKVK9ZKVWDAPAINQYCYHOFGOUCFRBLUYB9HGVWJPKQXLXF9JKQJLNMROEGCPFGXUSAIJNQ9YLVCCPUDJNJPNWLMYVDQPQXBTDEWMQVZYWDVKBTVEF9VOKDVHVZDZRWFK9WTMYU9ATRQJSKFDWXNPRDHQAYNGWXYFDKSIHFNA9UGGMVNCDMTZPEFBJAHGHMMDHEQLA9IGVDUGBQHJQORJ9ZWNEUJPVWVFDXKWDWLUHEVE9PYVYSMYPSYU9ZSMACZUAXCSAEJNVHOJGBRFYARSFZMYESXLNQOAB9HOH9VGQJTSAURKCMPYUFWZGEPHP9PUVFNHSCCHVVTGWYHLLVKIZCROMYQIYOCAHPONKRPEGJPKWTNQ9LIKTSHMKPESEZEPJETUUO99JDHXAUJIFEZCJLDGDTKZNLNEUSFYBWXHJILLHILNRXZKYAJDYYPNIJHIYXZ9RJKAGYLEVKVQDAIOCHGEKPQPCZOLSDXNFVFBDVCKKN9ZXJDSXGC9MMLCZGOBGDACJAHCOKJBDSTGANCAKNEVVTKARLTCPFRSGPPFPDIEDMCUEEFOGWUUXXQEPULAIFOEINWPDCMFXOJXCLQITCCMAUUDLGXDJDLVMOFEADOWJDVIXBBSINQMLGXGILLMHWIVHDBDTIHWOEETQZPJOSSWNWJYYGEDGMSNNJUDYTOATBYCACKRICFQALDFLDZGKFKCCLWZIKGWOGMEVKOITUUDCZCBQEDXWYGV9XARB9CKWNLCKVOZBWZAKAMAWNMLVGYDFQOARCUN9CA9YAKIZDBSFZXZRZJABWZLZGGNAXVOHZMXXMTCQZWIMSYYGS9QA9KLXYHLUFEGIFUVLAHKOINTMFMJZFUUWZKRT9GZZGSAWWCFUZUYNJXPWPKKYYZVUBCWSMWZFJEFZHDTLVACXIYPPFYWCFUDHZGTSOJPKNOADZPQPMYCJGSZRKANUOZRGKD9UAFAKUSDDPPPJORELCGTNZZQDBKZYWUBKM9GKGIBOJANBZFQMZKNVMLNZPQFFFVXYYUBHXQPBQZHCCZIP9Y99PIHURWQTJIJMA9L9UQIOMCWJLVCFTRQ9DNOWJKBEAANBCMLXX9DFYVDLVIYWAYPISVVYSMHYHHQMOVHMHLWPDWWMFYBUHNVSDU9BP9CSZVCLU9MEBETEJRNEQ9XPNGRPCMQSRNBZA9HUWBRCLEJEJZYVVFWGY9QMGCLYRVC9YBBKVDQNZDBTJGS9YZOH9XLNQFKPVWHEFPWOWOEPMRVBATFBOZAWIYVTPJMBCCBVSLBZTCXWQNGWLPCYXWHHHVUAAMABPWGIXMPIRPCWMKRPY9VYEGAHSIKDJT9GEXSPKFGUZWJBKRIGQSYUFFIRXWXFENFFHDHYROGHPVFHUXMMQETAHWJGJSVUUV9Z9CPWUTRDE9OYZAJXGXEJONLEUNSACYMXQBIBGELXFULFBU9AISG9IQSPASNHU9OWHILBVGYJAMEYMZOKMNFDBPMMORLENRVUOVZEBETIASNTBOHXUWAPAVJCZZGJUYAPTPREPLJOEW9OTD9DMALQOGUDZAKOYHEFALMNKGCKWECEQMSGDA9VPVHFEMYEVVHC9HVXQKQJABAYEGLFJVDWNCSOOPURHMYPGSWKSDFMCGTSZRYVQAKMATIRRGVIYHHTXATCXQHEJWOJMEVDXV9QCRGTGYQYDWCEJMRGSCHZOTERVNBDNCKZWUDZPXOOWDUXZ9GYQKNVUNZKUGOMPWDWGTTXGJIQ9XUOGKCEWMDKFYXIBUSAUGYJYJYUVEMAIKEXPLEBKMUEBECVIGYXJUQLISTYSXLQ9RNIHWVDWLXCXCYCUK9TAGBBFXCIYLOPRKZCKTDSMQVCUXU9WQCQ9GKPCN9PKUQYUAJDIAJKNTDWJH9PPGYGN9PVRKRMUBZZCW9SVGPLRRZDRADYMWGXWMWAFWLVGKVRI9WQGZIUHOXQJUCIBXQBB9VZNXDKT9ONCYCFZMTNHPJMJPUJZDWSPQLRAZNGV9QTKNYBZDSUQIPLVSQZHEOLUPFPRAXRHJSXADGQDBEVNPMVEC9TCYAGZSZASMGJLRZYWOKLUZULZZXTCBLZUQND9VPFJJHEDLOTRINEBFMPWSGDHKEOQACITVNPNTXKKQWCZLC9PYUHGSQEATWZRGZPELGOWUYVIHDYQPGJXAL9O9XNODA9GLDBYXXVPURQTQSWXZRWYRHOIQUUCRBZYTKSGSOMAXHRZSTUP9HRIMHNIBQJFCUBIWHIXHUWXATPHSVIFXMWJKKOBFMICGJGFJKLDPCVLFGHILHLIKFSDOPYFHEWVPGQISJQHKV9HBLKWX9JGDSUZWBUTPOHAINLBFNMUTEQ9NFFBKFSCZELK9XIXFTDZFQIYVIEULYEOEWMQWHZJZQR9SDBIOCXEPHXMCWCQSFCSDWY9QTJRDJDRFVJUCPBMRHIADTC9KZNDNUVDYSKQEDUX9BLFFNPNS9OCQHCHNMDYBRTOJNWAGWKLSTEAQHYCYEEYPKRUFPTWEWXMOQE9UKZYXHBFBHOHNDRYVTW9JKBIUZFF9OOFKFHPVCFYAEDUYSRMKKDCEOGNIJQYCYKVVUHUZRWMJFHVGTYFVFNXOIRXXZHBGMFQJEBUPSBKLAY9QTJZNIWMHYIQLVWF9TQMM9ZEDDKCCPEKDMJCJCKGDICZKINBYZGBCEUXAFMGWZTF9TUMVAEELCKMBESECIMAAVDRJWJBYEWDVB9HPVWNYUPUMOEFJETRHKD9N9KOAZRBFOQCXXMUBFPAVWZEEJ9FPOTLMLEMZ9EEJWKYMHDVXYFTKLRRCHWDWSCJTODJWIHWMFUTKGFLPDTFEYGCFAATPX9MTX9YCFWXWRMLZBNAAFOZMYUQZ9JYUXFQUXI9XKYVCTL9BIKJJPSVILKNDOHZWQQBG9QINKZPVG9EDU9WFVUZZQXZTCWWLHWIFQW9ECOJVGNYZFPXQAMKTVPEMAVLBQKUCBCQVFQFBDKATSOZGOQZJUKMOYZYHKECFQCR9NFEYCLFKFUMTSFZVYZYBFZQC9SAYIXTIPQJSKHTFEZ9NKPOYGRSOXROPRPGEJH9JPTLSI9VWQODQQZMAABCN9NNDUNO9WGWBSHLOXMTFWNTAFXAAMXBS9IHOPEPBRIBGDLKFCTEPSQWOZVKWJKZNGSTVYVJYKPCBUSIOY9FRPXCVBPCFMSKYDDXKYJJWMXMXDPZNAUNCKRCWDIHWGZUMUPMRBZKHSXEZSWWLXXVLLQBSVJFQWNSJZIA999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999";
        const DEPTH: u8 = 8;
        const INDEX: u64 = 4;

        mss_wots_generic_signature_verify::<Kerl>(PUBLIC_KEY, MESSAGE, SIGNATURE, DEPTH, INDEX);
    }

    #[test]
    fn mss_wots_curl27_sec_1_signature_verify() {
        const PUBLIC_KEY: &str = "ECFTA9SVHYH9MRRKJHQCBXNQKDBNGCWWDUAVILCOF9LMJNDPZLLTRYPKNHPVLXJYGGAXGOBYHZHGLNXKE";
        const MESSAGE: &str = "MMMMBNKRJWVAYO9ZCXYAODVATBOZPAQVLUH9FBCMQQOKTHKXOLIASYMMMMVRNNIQQAPVCUTMUVZAMYJA9";
        const SIGNATURE: &str = "XERUGTJZGCCJDUFSKVTNSZAENGJ9XBGRVVPVDVVWZMLQERHXNTSE9UPP99AIHZRUATYJUAVLEGAXVFVDRSFLQGSLWSNKKPBVSYXEMNAU9W9NLKYVSN9OLXS9URJFCR9GHINRIHDEOJNUDHADUWDFCAIESB9LJHOHFGWYYX9DDNPSQEXKGLJLKGALKTEZWOLIKJLWVTYVLUHYNON9UYKBXDKUTYJMQWHEZTNYNGXO9OMBLAPBDUCVWWVVAOQKNVLMZFMEFAX9MRZHEAFOPVBQEFHFEHXILKPJWVHZJUGGPQVHWYCRGJXMQOSNEHOZBKCCAHMRGZKBDDFSHJRNFSLXHVO9RQWJCPOBV9JUFZHAEQAEBIAQWNZNXHRIFKCVKOJSYTSACRIWJKLVIPYHGS9CGXFPTBBOCIV99O9GXZJAENKGLAOMKFBTXJRZOTWSMITMMI9GQFJQNYRHWDO9KUHHKDCRA9Z9MUBMLLPBOFSIGLAQAMHSSJTZSPTYJKPRJDVZLWGHFNRXSKOLVJLCOSLMBAVDMIUJCHOOAKJMHNRPHCFYTRPGIPIMNAYTAAKOJKFSXUMOYEZPWMXUGFJIOJVXQMGIXYKLOQWOCIOLIZTTOKVXVDRQAMBJVOQBDQHAAKDJKFRHXAFMPGXNLBKQIZUCTBBAZCKGCGOPXJHQSKGRZWTFTGALKESETAQKSXZWWKGFMGPTQFWHCXPJUN9MBTAVSAQSLNKVZHUKDDCEKLIEZJTOMBSPWIFUBTYCKOPKITSOEXENX9OHACHEH9WXBYMVSTYSAHEDWGTMGF9YKNFNJZUKJPYFEQ9HKIWNELFBBIHNGPUQQTCNAMZBZTRVDMQYJHV9LMRZZVVVKNTCJMDEEGQFXHRDHB9DMJDXEQWXLPPWJRIBYYXTAZNZSHL9RLFASMDLNVCZDEFIRDMEMAXQZMTPRWRWCZERT9EWUNBI9QRCHICMJEWVBHHRVDGSZCATFSHP9WFBQFMNPDDH9KYDVXIBVTOEMVTHZLDNASLKFDNQRXUNADCYOZOHBDQQPCHRHSNCAEANGSJVFUYLEJWCBF9OHM9GFRGKAO9VCJLDGXRCPECEOIKZPPFMDZCXDPCMNTHVELWLMIVAVZCZUGLFAZTLUYZKOIRPCPORDI9TCZZJU9XKZTCFVUGGFSCPKTKUGF9FJWBGQFJZQUT9FALIOJVEULADRINQHUQWNPWXSDHTPISNMHVHVSRZSTOIJUGOZBVPAGPBJVRLSABLMAH9OXG9CCTMRX9AYNUKBWZVNCINUYNZNJPUCJGQCQJHMHZOYFDWQEWRRYMOLUCBCHQMCVTPWZASBYQZUFUUKCYNVXNVHHAVVYBQPKHHKQWIJBRKZDZZFUDU9GJXUCKLXVVNEONUQSXNA9XUVCHOLIPHWQ9KBVIIGAXRHJBMTLGHQQSMMACAJSMHKZXECJPG9ELQIHTPUXDGAFYQTZACZMNUMDPSNSYUOKIUAOXNSJDTTZVAWJUQZITPTWSETMWXKEGDOKXVFUPSGUDMZZKRFPFWIPDFAJFFELZOXMZVJRGDFJAKCXQYPKVPNCENWJZVDOKNKJPXNGOP9KBP9RJEWTNCOABFJEOKONQSCCQRR9XCSXKTDJXVYJAVNUODHOXBZEUDFFTVCWHUPYV9RJZNVBAWCAL9XSGSMNFNADOAGFVEOXIAFRVEEFQKWEVTUVJHYINOSCHDWFSHJDATPUKJNBVFGKUQNUJLTHVNGVMXZWF9OCYKVJPKLME9EIWCIUBDBNHQJUMVXQJDJJDPJWHMVN9UCYKALOUSXZPZOGKVTGVAODHUTYVLEECAGNBRHBTQVOIMRGEHD9YJZITIEGRTDOA9FURPNRBDQAEPMLLUIHULV9AE99EADNBUYLDUKIBAIZHKFYJKGDQRKYTDZKOPCXG9WWBYOBZZBAHBOZVKXLPDZJRHBCSTCHMKSHGGYAXEVC9LWMZQISIJXAWOUVOULIHLAVTATZTZX9CRWJDYSGRPQVEBSDFVL9OAJLAHEISSYAKXKRTTSNHZPP9MDRCCPAIHWUYTKKJN9MZNGDJBDKJDEXYORFFJHXKK9MNGHPLYETLYGYBCJTFDPZGZTGICNEY9PFLWGIIGJ9CUOFAEXLHJNLHCTUYINWOHXAYJT9BTPDTFXIYEXBOWILI9AIDSQUVINLQMLEGLIGDYLWSRVCSRDCI9JTVKFYZXXVUPRXXCSPOLCHXQOBDCTBAGSGVVHANITAKBBMSWMWJ9TTTNF9HGMYXQEEDLDKXTDCFYXYLJJALBRJ9WMBYDBFGBHBUWYVDAHES9VSKVW9OXMOXMKNHAQPGRLFQRRGXRTTFPLYRZIYCDDJAHGTOLCUGHEHODTMKWUUAL9VFCISRXUKUTSZBPKPJEFSODZIBATZRPVFPGHDVXKOMIGTWTNYQKENYQKJFDCCRALOPQKCNZGQPRXQCAEYPZYZFSQMOCSLRTNTQWRYVFGPWMYKSUMKMMXCHPWRMF9QWKQJYYPKGJUJMBIAOQRII9PWKXUQOLNPCNKBCRDYM9FCGQ9BCMLIRHLIUANSYXBLZTLWGVIGJLWDWEQVLATWF9VJSQAKYEAZ9AAJMYEMXPXZDRXOOFRG9GGKARKNZZRTNOKZUBLPYNLPPLZMQPKACCHYNSQOBFXUYVWDGINAMIEODYZ9JTZQNRWCLGKCUBEGTKRIMESUPGYXQMIT9WWKQRVJNBOMXDOHBIOCNHWAICO9PQADCIEWLMFVZK9LXXMFWLWZ999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999";
        const DEPTH: u8 = 8;
        const INDEX: u64 = 4;

        mss_wots_generic_signature_verify::<CurlP27>(PUBLIC_KEY, MESSAGE, SIGNATURE, DEPTH, INDEX);
    }

    #[test]
    fn mss_wots_curl27_sec_3_signature_verify() {
        const PUBLIC_KEY: &str = "ROLHKXFNMSN9WWAWLWYKWXJUQ9BREXTKOMCZFT99JOLWNWBPUYSCZPLSOSOPICLKXXSDRAYEYRNTTKTNI";
        const MESSAGE: &str = "NKWSPOYLCE9NMZEWZBRLMZISBYBMMMKCOAUU9PQRHZYZPFVXPQHILAUQLHYKRPMTTIXGMSCTJWFVNMQX9";
        const SIGNATURE: &str = "JYIDCZPR9QNEUDENZCQQLBQNWPMQ9O9VOTTZNO9RWIDRQRUWZNURKZWLMHZRXUEYHUFCKUUCVMWMGBTEIAMPXQYYTB9S9YRHBEDWJRVPYCMFWTLUP9NWGSYQDYJAJMOGXERUYSGKFDKTFRWSRPGXUOAEPOFN9FURIJADBRKEZWQNDNJALFAQAZL9QJVGDK9KNIFHVLQI9ARIC9ZBRARPHFDGZGNU9SHCFWWCYUPXQRQADWBYMHNEZXXNDJYY9QTPXATBKHXBNHUNWCPARKWIJ9OLWVHJVU9SLVVUGECSZH9VDSCZJRJEIZVTSUVRLMJPKLQSJW9XUTAAXXCQ9FZVHKKFBXFDHUXUCAUGKIOWL9ZFQCPQT9WRWKWRPXRYXCJCNOKGRNBWBVYXISEIW9FLZBAVQ9SQNIXWZEVBEJSFD9LEGOYJMMGKWC9AESFF9HXETEHTXT9TULC9LIATTEVRVWRFKCUNJOOOAOMBZOMUYDUHVFWSPOTJACZAFCPGSZXRHPUCCNHEHHIFJMOCV9CXEPDT9G9DPWEH9LIDZKAYBXKFLTJQSUMBBJV9HHQQCX9TADOIVW9TBYGXSPDIATJEUBJK9OXBMIJWIMOC9OFUNIBEYSTGZCMNWZDKCPALIGEUAIQLSDEDUOXKJJWXXGEC9DTXWYRQPPGOUGYTSBXVOXUJIILLNLLSVRSJCFNUKEBINRLBFBOXUWPXYETMIWSGXXZMOJJMHLHHDQXZETJSPUBBQMDEVRQAUTLNLTNFEUURRGVUDJFESHKRJMKACUPPIYGQCZQVWZUOAYAAIOZVPRCYUFTMMIEUCCJZRQJDCWLUQIJYYSGTGJEESLME9DTTFJL9XHYDDSLBIEXYYQZ9BMSZMLIN9ANJXWOT9LHMCID9LIFKJZSAQOCWPBDCHI9BYF9SGKZFXZDXIPGDMEUDDRIRQUSYDVEYEEEWBAHJZKNZXVTTPGYVQWFSRBWZEVGYFWEQJONFGBWTLKKVPZ9CNHEYDKPK9YW9JBRXMMYQVHMHRUPPQQGZQRPLKPXRGSX9LQMETBEWAZKSVLKJFYTBZXVLDCKS9TZTZJFZQJPTB9RZJYJGACMFDPCOBSDMXEPFJDJXGEMLZKRLXMOQUYQACLMSUALGDGEFTRIYZBNYWKGKWTVXQXCXVDYRBZHMANNYATAPQKXEALVFEGS9MJZCTXOUCBNHKRWJNTKFMPFOI9LGEI9YEYAFYA9GRCTCBUP9ABJUABWZQKV9WETMQACUQKYSRXJWNUYZKDZQISRQPZOEUQFYYOTPWMZGYFQPTIW9ZEGKSBQYUUNBTFM9PGRKBPPV99TEXQAREHRUCWJ9ETABXDTBJDHPCCLEFSHKHMZPENXNQVHMHJAZVIAWR99UMFYGNBFMKIDVBMSBXANIXMEIPOUHPWBBXDMCETESPCB9AOQNGZVGKJHSSJFIRHJEKWLSCNYTIQXXZCGSYECRIGITGYIRRZRE9YGUTTSQARCCZBKBDJVZQLQDOVESVCXRSFXAAJAZPNURMOEUGMCRWBXNIFKKSOCPUMWP9UICRSZKZJNRUAV9VEJMOLLP9TECFBPINGTUZBZ9ARZIJTIQ9MTQOPNUNVIUHIXSDCFOJEWYJUAYJSKEVDQYXGH9IDMZDDHOBQ9NYUTPDRDPSTEIHXST9MPEQUGGMHTHECPLSH9CPPZHRDWPHVZGHCYFT9ESHTTPGRUBELXKOZHTG9LKLTPFGZAAGQKPEEOPZFKOWMPMAUSGBDYOZCHHFZZENDFVSFSKNCT9RQZXIIKTBP9MKRJYYMUDIPGVPFOXVFRKMBNRRTKO9STAAXVBHHQAWUGGYBUSSRKVMLLVBNLJOW99MJOUPZFXOPAMJFQLBURBWYIZLICWCNKJBCLQAAOSEGKCHJTYXMAFSRECUR9RKJW99NNHMOHWAEOJKTQCOENF9ROEYBSEAYFYNVEWEL9DTTAATCKVHEBMFMZPXWILAIEFNXRIWCSJRLYGZXFLVASNGSGKMQZFQJYDJUBOETECNGUQQXJXZYLMQCYKWXGIZTOPCIPYCVHAII9PTJLWAHHXORFQATHMTYLRTFFIS9WMNSLNET99QSCPLKXOGYMLEHRQQGFDRLHBRPKTUJGLKYENBIJGCTOSBYQ9GRYXB9OULPJHYTBIJPUL9YJNQVOGFZQZGIVKUYGOCVZDIDUIFKHDFXGMESJHRPNKSKJNJZLJNXUEKJGQZXWSYBGPEHPSFB9BXFHMMSLUPAHD9KAEMASLVNBOQ9NANOKHQBLFHIUIKKVBROGBYVABWEGRBYOPYPOJXYEKMEFGYBEPUTKBJWALHRIZILHATXXENUPQNQDOCBGFKEXNAYBHOQXF9WZUUPADXGRIRHLTZZBDFOGH9ADOY9QSMWOUARWMPJJEJZZNXAGGKWYBNQNZBFJFDIJKJTHBPRHI9UUGDVBPNTDXVIDGAEFBADUQRHVLEGZOTWBQIDVUVZYOOHXNTVOUUEMG9YTTSAXSINFPBIONVZSEFZNHZCXNZADLXXHNBTSEPHUA9IPUXESBXDLFLJWKJBXOX9HKQIOFKFQYJIMTQYJSBBHGVGTMTSQUTOPYVGSJPDIRRPIXLYASBAJWJIDTTTJYOXVSHASCFOTBCZYCTCCYAZB9TDLHFL9BJB9TGBEQRVJXVWSNNVJGYQXUJLCB9ZVRWCUVNFMNZZ9SBAVGENHMKJO9MGNKYRQTPFCETOLPDCPTWGMZJHKRKI9QWMBTQMLHI9FVHSYWGZQUIDA9C9NDAKGKXBIRZGMSSZAFPEOKDPNNUQIDDOBMSF9HHARDEGAHSDHFGSECSVTIGKKUJRCNGQNIYWMMULPNHZNJX9MWDIESXPTCGAAZQPKUPSLZGCFBXRUYSDYTQDJXPYBZVAWCXJTOVA9EJZG9XN9RAXAW9PBVTGMYJUOWLHMZNUXCJP9URUGJJYHKKFVZNNKPSXNSYIS9STBETRFKUM9QABQHFPFKTQLONWINQEYEJKWQASOBWBRCZRV9VDQJKMOTYLPYRBZANWFERYMPBOTDFWHFZJKFMOPVWABJWZDAFTIZNCZO99UBGHGFTXHZSMDVCXIX9UBPTYQYPEIHFXBONHHZHLYRCNVAEVTCZIHZVZZCCDMCPUFZEZVBWH9HSBFJATDLAGVOCGLBIVWJSRCLQQWTWNQSXHKRIQLLG9BCRSHNTSYRYWJJXVZDOHFSBXTOSFTNKSVEVPOTDYFYGHVAMUITVDSXDAARRFYETDRW9NW9CTOFNOFURWWTFDWNKVHRODCCPRFGIFPZBBFSZZHDAEZ9BOFTVFMUASHFRCVFHSDKQKILHUUMUK9ZZIHZTREQGXLLPHDNUOGOVNUEGYDHQYNQ9XWHCRJ9IWXJUKQYPDSHWCAVVWZNYSVGBPIGFFMT9PGHKXYCAZVCUOQVFWPOXES9SHNZSEBQAHHZRJHMBGJ9EJAMZKFQEIFTSTCOFBWEVMXGGJZYNFOAAFFVRXIMFCBRNFHIXMDKLRJIOJGXQABDVXKKICKXMNDWIQOPGFYSRNVWWKMZTC9CQCCPAJIMAWQYTSS9SAFOWNLAOSXGR9WI9PYHBKGTTQFBSSZEW9FHTMBGFIWFCEBH9ZPU9YARCLGLIDDKPIHHUDOSE9YINTJHWAZOWACAJHXXV9ZWDKQRHXDKFCLGBT9TNIKYKNGEIJEKRRI9SHKL9TFVTOHMKRVVWSZIJWFLDLJHKDWPQVOEDZU9DF9JXNFVNUBI9JUBTTAKKR9WAF9AGTPVFDPSOCPXVSYJMHGGWLUBQQTJVJGPAFBWVYQTEBTENKJARVSJ9VCYVCLSHIRVOQVHDRLR9FSOWJ9FAKCULBQYXARKPGVPXFCCDTZEYPMI9OZUBNNDPLLKQW9COFNAKLUTJQBCWAPYICQAVGYAQOYRSWVHKDJQVPGFZWGDFMTPUIOTKDB9FSJSXPFYYNWRICNKUROBQHLYLPJIQBJFGHIUVQYKDMQNHHRXNVWXE9TZKRFNNVHCKZNVICCSSMGSQZTPVTCWRASVCNMIH9S9VGUPAHMSB9ZPUOVAFDPIHIRHMXKFDHJXUGDFP9IGGL9CRNNMVAL99HXZB9KIVUYMKAUSLNPBMMX9VEBGAKXFTURHKGYR9LQRAPGJZQAJKAAQTFEYXDFNLZKJKHCDPEOKNIRXKZIMPSTGKWWVYREWXUEAPPHLSVGSJTCBZWCCMXRSAKWBQTDDSWUKKQBSKANV9R9BAXEPMNRH9MLSQLZWPTUMD9SDTWPIXEGLYDWFJILKJFZE9EHXSFWWDUWWMCIOPXMFSPHBCYOTUWPDLQOFU9EUT9FXSCQJBFKZIJMKYYQGVVGGBPCJHKPLBYSCZJDEUKK9YIXESUEVJTUB9UGFMXTUVIUUKAMTRJXHUSIIBVAYPY9RBQAKFADROQYRJVSTWZMVPQ99EFXFIBRBGSOMPQFSO9PNEXNOTJEOI9F9VQJF9QHZUTLJJDBSEMBKVUPBEVLDIGBQIKYIPDBARIMCEYTW9LVW9PQMVIRRCIPOE9TMCCVPUO9MIAHHBVZEBHRUASOSSCKFJIKLLNGRTYGLYCMU9FPLPOUCHQEYYUPQXCMZWHXCYLFAVPWHXFYYKQVLSCHESVDUBSRHPPOOOVRSVQFCMBUYIAFNBERKQVXWLBKHDKYNWXLAAOCSKBIHHXZ9XRCJZELNCEXFWUKSHZLRXJAHFBXNEJOTSYPSTVHECBWKHDTRENPCDGPAHIQJMQKJPDNGUUPYPQRHJORGYRMAAKPVYZBP9CUBIJQQRQPSPCCVRJZCUJZQNQCKNPAWIXZHMWTCWDGPLAHRUNVS9QVOLYXEDZRMNDSCRLISGHI9SNCAGPICRAIBQNDILYQIAZO9RBKBYIDOAGKFOEJNUSEUECREDOYPH9EZIQVFKGVJPOHGLXCFUKJWDCKEQAZIKKKIHAUSFNNHOFZLMVSFDJPZYNTWLHLFBTASMFQGTODHYPKGUKRAHAZNRILYEBSYEFEUJWOYIAACENBHGJFACRWDZXOTOOBYFOZS9YSPSRMYZMKBMYSBMWCHJV9LXQQEASTNPVC9USWEQBWHDGXNNYMCMUIHWANCPSRLWPXNHDOLMTMXBLRWXWK9ASDQNGFAJJPNFMTPQERZF9FY9ENZSKDQXXSBABCNK9FSMTEFLXHHXIWSKEXY9EU9QYZOHXGHAYDVDZJH9YWAXXBSUURBBIXYSIEAOAGBVP9F9NJBPN9AJAYONVNRIWOUHAZDGZ9NKXHOIWBTQLXRRQCHG9DQNZYXPXKPPOHMNMWTVEETFLKIMKUDOKMKCW99T9OTNTXBYYHUKUQUYJPRCPDMWIRHYMTXHQWVDHUBXYXKAFU9ZDODFXAZCTVJPTZALFKIEEX9KFNQXORIVYSNQDXEDTQ9LCCSWGSVGJADOZWVXYPVYGV9MSKWWHQFSWSQNOUWVWIKAVDOCTVNTXNGRAO9TLMPRUR9FFZWTAGANOTKNG9OPINIYDNAOPYNLGQMDUSGYEZZWAEYE99ZWVDJHBYSOOXFYFCGISWCTMFUOG9OJVDXEDIOOEF9HVZWQVWANYKNJIYARMDHNDWTZMNFOFGUVVETUBPIDRRZCGYDEKVJQVUHWJSQLBN9VMWONIBABYFLHDNYSNJMLEZKBAVRXYWAYQIWXBTSMUINPOLKNAWNDEWPTAQFYRCLABXL9GLSJBNBWXYDLYOLYVNV9UAFLDHNAELXBYQXERGHAYDRDHAYEUXXTXHQYTOIHDLRROCFJCR9CVWJPLXKAEFPPHWWQPKNZWSWI9WWYGWKUKXMPLZSWDJGRRGANQF9YVNFGIYRCIDHSIU9TUGNHRRTCXVQFNVMUM9GRJFQKDPKCTVQRBIYVIVFBPLVBNIBTKBMGRDDPEYQNNWYFBZEHWCLNZMKZUXQTGOXRPNWYHCWSCHUUNYCTRWBZXVJECIMWTAUAJFTC99AUNEZIRJEPYADMYNW9HFQHGX9QYMGRCLUTQLIHOGAYWRVJVOJZERUPFTOOXQCFEEIXHFBVDITQCPMCVXDNWHIBESNGDBUJHZALMVOOTH9YLFMEZQTPUQBVD9JPKUVBDDMNLRZOFTNLNWKEMRRYCHCXGRSPWMRISRUHOTOJPUCXJZZMZRSBDNVDNWZQLAZWTVJPFUARURHHGOZPBOICYQHY9EIBSZFZSEVDUGKETNVLTKYCGGHNYQYBF9OWDHMOVT9FPYWCVSMBDRTGTBVGPRMVFVWLAZISLEJWLHZWDGYWRZBEHBMNOELYKIMGHGBISMKNLLISNOA9RMFBLXDKNEVVSARAZUDYPVMGOKMPWOIKDHWCFTOMEGPPCALOE9NC9DEFVMUOFFQQJDIPAXOESW9LOYANYLHTVYQBAOCJYDWJKAMPTVRRXAJPPMFFMYNXHBBLLQ9IMMRSCSQXPAGR9BSRZXGZJYVNCWPADSHXITMPBBVMDUEYIMKPJOWBFIUZJYWHFCKRPEU9FZIYNSRZJP9ECGKSYLV9LEFPPEFA9VYBV9TZUXXVUYMGDBTCDHIUTWMCB9NGJBPXGTPNIOHLWNTQIKINRKCOAWYDIQLPYPQDSSSLQDIDMKZDRFHZZACOPWAQJLQXSCAJXNAUPPNLH9XYYUFRNXZS9XKXWUVVYVB9YRSGOLHFGPBKVYMVHCNKPBEAGWWUE9ROVFYHXAUZAOPLNNQQTQXNBXWJMJVQZEQWSZAQYUQHKQGJTSHLXBR9PUVQUWSG9GUMVVXDKZOZEJTAEJHMTOOWUWTO9NUOKL9MIJDSW9HZMLWEJYJTKHRDVBPWUTKJI9GBQZUQRMNDMFTDHJJKPGZVMNYAKXGHQIQPLPSMJOGIIMVHXMYNPTFYKEFWYHBIEASGBYUD9GFIBUIJBKNJCOAGNDDKHGMNWFBRDXUVIZBBENU9QQAZU9SRLVADEWUIHKKOCERSDEGVFKDXYAQDKGLVKVNWXJ9PVIUD9RYYATRM9AAKSPVMDGBNIAMZPWXALNDBHWUBNQWVMIAJIFCAEOSETTEAZ9GBCXP9EPSKBQMZPDPQXOKVRBIWJTMRLWJAJJFPICRQFKGXYYIERERVKRDYTLUNLQNJYHFEDRRQJHYSGLANFGBVAZJLKOYJMJHPJACMMMCQNPZJEHGRYLXSTADGKLVXKSD999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999";
        const DEPTH: u8 = 8;
        const INDEX: u64 = 4;

        mss_wots_generic_signature_verify::<CurlP27>(PUBLIC_KEY, MESSAGE, SIGNATURE, DEPTH, INDEX);
    }

    fn mss_wots_generic_roundtrip<S, G>(generator: G)
    where
        S: Sponge + Default,
        G: PrivateKeyGenerator,
        <<<G as PrivateKeyGenerator>::PrivateKey as PrivateKey>::PublicKey as PublicKey>::Signature:
            RecoverableSignature,
        <<G as PrivateKeyGenerator>::Seed as Seed>::Error: std::fmt::Debug,
    {
        const SEED: &str = "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN";
        const MESSAGE: &str = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";
        const DEPTH: u8 = 4;

        let seed_trits = TryteBuf::try_from_str(SEED).unwrap().as_trits().encode::<T1B1Buf>();
        let seed = G::Seed::from_buf(seed_trits).unwrap();
        let message_trits = TryteBuf::try_from_str(MESSAGE).unwrap().as_trits().encode::<T1B1Buf>();

        // todo try with not recover
        let mss_private_key_generator = MssPrivateKeyGeneratorBuilder::<S, G>::default()
            .depth(DEPTH)
            .generator(generator)
            .build()
            .unwrap();
        let mut mss_private_key = mss_private_key_generator.generate_from_seed(&seed, 0).unwrap();
        let mss_public_key = mss_private_key.generate_public_key().unwrap();

        for _ in 0..1 << (DEPTH - 1) {
            let mss_signature = mss_private_key.sign(message_trits.as_i8_slice()).unwrap();
            let valid = mss_public_key
                .verify(message_trits.as_i8_slice(), &mss_signature)
                .unwrap();

            assert!(valid);
            //  TODO invalid test
        }
    }

    #[test]
    fn mss_kerl_wots_kerl_roundtrip() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        mss_wots_generic_roundtrip::<Kerl, WotsSpongePrivateKeyGenerator<Kerl>>(wots_private_key_generator);
    }

    #[test]
    fn mss_kerl_wots_curl27_roundtrip() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<CurlP27>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        mss_wots_generic_roundtrip::<Kerl, WotsSpongePrivateKeyGenerator<CurlP27>>(wots_private_key_generator);
    }

    #[test]
    fn mss_kerl_wots_curl81_roundtrip() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<CurlP81>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        mss_wots_generic_roundtrip::<Kerl, WotsSpongePrivateKeyGenerator<CurlP81>>(wots_private_key_generator);
    }

    #[test]
    fn mss_curl27_wots_kerl_roundtrip() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        mss_wots_generic_roundtrip::<CurlP27, WotsSpongePrivateKeyGenerator<Kerl>>(wots_private_key_generator);
    }

    #[test]
    fn mss_curl27_wots_curl27_roundtrip() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<CurlP27>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        mss_wots_generic_roundtrip::<CurlP27, WotsSpongePrivateKeyGenerator<CurlP27>>(wots_private_key_generator);
    }

    #[test]
    fn mss_curl27_wots_curl81_roundtrip() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<CurlP81>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        mss_wots_generic_roundtrip::<CurlP27, WotsSpongePrivateKeyGenerator<CurlP81>>(wots_private_key_generator);
    }

    #[test]
    fn mss_curl81_wots_kerl_roundtrip() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        mss_wots_generic_roundtrip::<CurlP81, WotsSpongePrivateKeyGenerator<Kerl>>(wots_private_key_generator);
    }

    #[test]
    fn mss_curl81_wots_curl27_roundtrip() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<CurlP27>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        mss_wots_generic_roundtrip::<CurlP81, WotsSpongePrivateKeyGenerator<CurlP27>>(wots_private_key_generator);
    }

    #[test]
    fn mss_curl81_wots_curl81_roundtrip() {
        let wots_private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<CurlP81>::default()
            .security_level(WotsSecurityLevel::Low)
            .build()
            .unwrap();
        mss_wots_generic_roundtrip::<CurlP81, WotsSpongePrivateKeyGenerator<CurlP81>>(wots_private_key_generator);
    }
}
