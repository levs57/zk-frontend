extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn make_tuple_impls(_item: TokenStream) -> TokenStream {
    let mut buf = "".to_string();

    for k in 2..13 {
        let mut ti = "(T1".to_string();
        for i in 1..k {
            ti.push_str(&format!(", T{}", i + 1));
        }
        ti.push_str(")");

        let mut ti_node = "T1:NodeStruct<C>".to_string();
        for i in 1..k {
            ti_node.push_str(&format!(", T{}: NodeStruct<C>", i + 1));
        }

        let mut ti_fstruct = "(T1::FStruct".to_string();
        for i in 1..k {
            ti_fstruct.push_str(&format!(", T{}::FStruct", i + 1));
        }
        ti_fstruct.push_str(")");

        let mut ti_alloc = "(T1::alloc_to(c)".to_string();
        for i in 1..k {
            ti_alloc.push_str(&format!(", T{}::alloc_to(c)", i + 1));
        }
        ti_alloc.push_str(")");

        let mut self_i_read = "(self.0.read_from(c)".to_string();
        for i in 1..k {
            self_i_read.push_str(&format!(", self.{}.read_from(c)", i));
        }
        self_i_read.push_str(")");

        let mut self_i_write = "self.0.write_to(c, value.0);
        "
        .to_string();

        for i in 1..k {
            self_i_write.push_str(&format!(
                "self.{i}.write_to(c, value.{i});
        ",
                i = i
            ));
        }

        let mut ti_sv = "T1: SVStruct<C>".to_string();
        for i in 1..k {
            ti_sv.push_str(&format!(", T{}: SVStruct<C>", i + 1));
        }

        let mut ti_sg = "T1: SigStruct<C>".to_string();
        for i in 1..k {
            ti_sg.push_str(&format!(", T{}: SigStruct<C>", i + 1));
        }

        let mut ti_cr = "T1: CRhsStruct<C>".to_string();
        for i in 1..k {
            ti_cr.push_str(&format!(", T{}: CRhsStruct<C>", i + 1));
        }

        let mut ret_append = "".to_string();
        for i in 0..k {
            ret_append.push_str(&format!(
                "
            ret.append(&mut self.{i}.to_raw_addr(c));"
            ))
        }

        let mut let_q = "".to_string();
        for i in 1..k + 1 {
            let_q.push_str(&format!(
                "
            let q{i} = T{i}::try_from_raw_addr(c, raws);"
            ));
        }

        let mut qs = "(q1".to_string();
        for i in 1..k {
            qs.push_str(&format!(", q{s}", s = i + 1));
        }
        qs.push_str(&")");

        let incoming = format!(
            "impl<C: Circuit, {ti_node}> NodeStruct<C> for {ti} {{
            type FStruct = {ti_fstruct};
        
            fn alloc_to(c: &mut C) -> Self {{
                {ti_alloc}
            }}
        
            fn read_from(self, c: &C) -> Self::FStruct {{
                {self_i_read}
            }}
        
            fn write_to(self, c: &mut C, value: Self::FStruct) {{
                {self_i_write}
            }}

            fn to_raw_addr(&self, c: &C) -> VecDeque<<C as Circuit>::RawAddr> {{
                let mut ret = VecDeque::new();
                {ret_append}
                ret
            }}
        
            fn try_from_raw_addr(c: &C, raws: &mut VecDeque<<C as Circuit>::RawAddr>) -> Self {{
                {let_q}
                {qs}
            }}
        }}
        
        impl<C: Circuit, {ti_sv}> SVStruct<C> for {ti} {{}}
        impl<C: Circuit, {ti_sg}> SigStruct<C> for {ti} {{}}
        impl<C: Circuit, {ti_cr}> CRhsStruct<C> for {ti} {{}}
        
        
        "
        );

        buf.push_str(&incoming);
    }

    buf.parse().unwrap()
}
