// SPDX-FileCopyrightText: 2026 The bpftop Authors
// SPDX-License-Identifier: Apache-2.0

use libbpf_rs::query::{
    CgroupLinkInfo, KprobeMultiLinkInfo, LinkInfo, LinkInfoIter, LinkTypeInfo::*, NetNsLinkInfo,
    NetfilterLinkInfo, NetkitLinkInfo, RawTracepointLinkInfo, SockMapLinkInfo, StructOpsLinkInfo,
    TcxLinkInfo, TracingLinkInfo, UprobeMultiLinkInfo, XdpLinkInfo,
};
use nix::net::if_::if_indextoname;
use ratatui::{style::Stylize as _, text::Line, widgets::ListItem};

use crate::helpers::{attach_type_as_str, link_type_as_str};

/// Collect and render all attachments used by the BPF program as a list of [`ListItem`]:
///
/// ```text
/// BPF Links (<number-of-links-used>)
///   ID <link-id>: <link-type>  <link-specific-metadata>
///   ...
/// TC Filters (<number-of-tc-filters-used>)
///   <iface>(<ifindex>)  <direction>  [direct-action]
///   ...
/// ```
pub(crate) fn render_prog_attachments<'a>(prog_id: u32) -> Vec<ListItem<'a>> {
    // Collect BPF links
    let links = LinkInfoIter::default()
        .filter_map(move |link| (link.prog_id == prog_id).then_some(link))
        .collect::<Vec<_>>();

    let mut attachments = Vec::with_capacity(1 + links.len());
    if !links.is_empty() {
        attachments.push(ListItem::new(Line::from_iter([
            "BPF Links".bold(),
            format!(" ({})", links.len()).into(),
        ])));

        for link in links {
            attachments.push(ListItem::new(render_bpf_link(link)));
        }
    }

    attachments
}

/// Render the BPF link info as a [`Line`]: `  ID <link-id>: <link-type> <link-specific-metadata>`
fn render_bpf_link<'a>(link: LinkInfo) -> Line<'a> {
    let link_type = link_type_as_str(&link.info);
    let metadata = match link.info {
        RawTracepoint(info) => {
            let RawTracepointLinkInfo { name } = info;
            format!(" {}", name)
        }
        Tracing(info) => {
            let TracingLinkInfo { attach_type } = info;
            let attach = attach_type_as_str(&attach_type);

            format!(" {}", attach)
        }
        Cgroup(info) => {
            let CgroupLinkInfo {
                cgroup_id,
                attach_type,
            } = info;
            let attach = attach_type_as_str(&attach_type);

            format!(" {} CgroupId({})", attach, cgroup_id)
        }
        Iter => "".into(),
        NetNs(info) => {
            let NetNsLinkInfo { ino, attach_type } = info;
            let attach = attach_type_as_str(&attach_type);

            format!(" {} Inode({})", attach, ino)
        }
        Xdp(info) => {
            let XdpLinkInfo { ifindex } = info;
            let ifname_cstr = if_indextoname(ifindex).unwrap_or_default();

            format!(" {}({})", ifname_cstr.to_string_lossy(), ifindex)
        }
        StructOps(info) => {
            let StructOpsLinkInfo { map_id } = info;
            format!(" MapId({})", map_id)
        }
        Netfilter(info) => {
            let NetfilterLinkInfo {
                protocol_family,
                hooknum,
                priority,
                flags,
            } = info;
            let nf_proto = match protocol_family as i32 {
                libbpf_rs::NFPROTO_IPV4 => "IPv4",
                libbpf_rs::NFPROTO_IPV6 => "IPv6",
                _ => "",
            };
            let inet_hook = match hooknum as i32 {
                libbpf_rs::NF_INET_PRE_ROUTING => "PreRouting",
                libbpf_rs::NF_INET_LOCAL_IN => "LocalIn",
                libbpf_rs::NF_INET_FORWARD => "Forward",
                libbpf_rs::NF_INET_LOCAL_OUT => "LocalOut",
                libbpf_rs::NF_INET_POST_ROUTING => "PostRouting",
                _ => "",
            };
            let ip_defrag = if flags & libbpf_sys::BPF_F_NETFILTER_IP_DEFRAG != 0 {
                "IpDefrag"
            } else {
                ""
            };

            format!(
                " {} {} Priority({}) {}",
                nf_proto, inet_hook, priority, ip_defrag
            )
        }
        KprobeMulti(info) => {
            let KprobeMultiLinkInfo {
                count,
                flags,
                missed,
            } = info;
            let ret_probe = if flags & libbpf_sys::BPF_F_KPROBE_MULTI_RETURN != 0 {
                "Return"
            } else {
                ""
            };

            format!(" Count({}) Missed({}) {}", count, missed, ret_probe)
        }
        UprobeMulti(info) => {
            let UprobeMultiLinkInfo {
                count, flags, pid, ..
            } = info;
            let ret_probe = if flags & libbpf_sys::BPF_F_UPROBE_MULTI_RETURN != 0 {
                "Return"
            } else {
                ""
            };

            format!(" TargetPid({}) Count({}) {}", pid, count, ret_probe)
        }
        Tcx(info) => {
            let TcxLinkInfo {
                ifindex,
                attach_type,
            } = info;
            let ifname_cstr = if_indextoname(ifindex).unwrap_or_default();
            let attach = attach_type_as_str(&attach_type);

            format!(" {}({}) {}", ifname_cstr.to_string_lossy(), ifindex, attach)
        }
        Netkit(info) => {
            let NetkitLinkInfo {
                ifindex,
                attach_type,
            } = info;
            let ifname_cstr = if_indextoname(ifindex).unwrap_or_default();
            let attach = attach_type_as_str(&attach_type);

            format!(" {}({}) {}", ifname_cstr.to_string_lossy(), ifindex, attach)
        }
        SockMap(info) => {
            let SockMapLinkInfo {
                map_id,
                attach_type,
            } = info;
            let attach = attach_type_as_str(&attach_type);

            format!(" MapId({}) {}", map_id, attach)
        }
        PerfEvent => "".into(),
        Unknown => "".into(),
    };

    Line::from_iter([
        format!("  ID {}: {}", link.id, link_type).bold(),
        metadata.into(),
    ])
}
