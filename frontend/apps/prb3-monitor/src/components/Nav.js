import {ALIGN, HeaderNavigation, StyledNavigationItem, StyledNavigationList} from 'baseui/header-navigation';
import {Button, KIND, SIZE} from 'baseui/button';
import {ChevronDown, ChevronRight} from 'baseui/icon';
import {PLACEMENT, TRIGGER_TYPE} from 'baseui/tooltip';
import {StatefulMenu} from 'baseui/menu';
import {StatefulPopover} from 'baseui/popover';
import {useAtom, useAtomValue, useSetAtom} from 'jotai';
import {allWmAtom, currentWmAtom, currentWmIdAtom} from '@/state';
import {useRouter} from 'next/router';
import {forwardRef, useState} from 'react';
import {Modal, ModalBody, ModalHeader} from 'baseui/modal';
import {ARTWORK_SIZES, ListItem, ListItemLabel, MenuAdapter} from 'baseui/list';
import {useStyletron} from 'baseui';

const WmSelector = ({isOpen, onClose}) => {
    const allWm = useAtomValue(allWmAtom);
    const setCurrWm = useSetAtom(currentWmIdAtom);
    return (
        <Modal isOpen={isOpen} onClose={onClose}>
            <ModalHeader>Select WM</ModalHeader>
            <ModalBody>
                <StatefulMenu
                    items={allWm}
                    onItemSelect={({item}) => {
                        setCurrWm(item.name);
                        onClose();
                    }}
                    overrides={{
                        List: {
                            style: {
                                boxShadow: '0',
                            },
                        },
                        ListItem: {
                            // eslint-disable-next-line react/display-name
                            component: forwardRef((props, ref) => {
                                return (
                                    <MenuAdapter {...props} ref={ref} endEnhancer={() => <ChevronRight />}>
                                        <ListItemLabel
                                            description={
                                                (props.item.proxied ? '(Local Proxied)' : '') + props.item.endpoint
                                            }
                                        >
                                            {props.item.name}
                                        </ListItemLabel>
                                    </MenuAdapter>
                                );
                            }),
                        },
                    }}
                />
            </ModalBody>
        </Modal>
    );
};
export default function Nav() {
    const router = useRouter();
    const currWm = useAtomValue(currentWmAtom);
    const [wmModal, setWmModal] = useState(false);

    return (
        <>
            <HeaderNavigation>
                <StyledNavigationList $align={ALIGN.left}>
                    <StyledNavigationItem>
                        <WmSelector
                            isOpen={wmModal}
                            onClose={() => {
                                setWmModal(false);
                            }}
                        />
                        <Button
                            endEnhancer={ChevronRight}
                            kind={KIND.tertiary}
                            size={SIZE.default}
                            onClick={() => {
                                setWmModal(true);
                            }}
                        >
                            {currWm ? currWm.name : 'No WM Selected'}
                        </Button>
                    </StyledNavigationItem>
                </StyledNavigationList>
                <StyledNavigationList $align={ALIGN.center} />
                <StyledNavigationList $align={ALIGN.right}>
                    <StatefulPopover
                        triggerType={TRIGGER_TYPE.hover}
                        content={() => (
                            <StatefulMenu
                                items={[
                                    {label: 'Workers', url: '/status/worker'},
                                    {label: 'Transactions', url: '/status/tx'},
                                ]}
                                onItemSelect={({item: i}) => router.push(i.url)}
                            />
                        )}
                        placement={PLACEMENT.bottomRight}
                        returnFocus={false}
                        autoFocus={false}
                    >
                        <StyledNavigationItem>
                            <Button endEnhancer={ChevronDown} kind={KIND.tertiary} size={SIZE.default}>
                                Status
                            </Button>
                        </StyledNavigationItem>
                    </StatefulPopover>
                    <StatefulPopover
                        triggerType={TRIGGER_TYPE.hover}
                        content={() => (
                            <StatefulMenu
                                items={[
                                    {
                                        label: 'Workers',
                                        url: '/inv/worker',
                                    },
                                    {label: 'Pools', url: '/inv/pool'},
                                    {label: 'Pool Operators', url: '/inv/op'},
                                ]}
                                onItemSelect={({item: i}) => router.push(i.url)}
                            />
                        )}
                        placement={PLACEMENT.bottomRight}
                        returnFocus={false}
                        autoFocus={false}
                    >
                        <StyledNavigationItem>
                            <Button endEnhancer={ChevronDown} kind={KIND.tertiary} size={SIZE.default}>
                                Inventory
                            </Button>
                        </StyledNavigationItem>
                    </StatefulPopover>
                </StyledNavigationList>
                <StyledNavigationList $align={ALIGN.right} />
            </HeaderNavigation>
        </>
    );
}
